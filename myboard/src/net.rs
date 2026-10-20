use cyw43::aligned_bytes;
use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_net::{Config, StackResources};
use embassy_rp::{clocks::RoscRng, gpio::Output, peripherals::PIO0};
use heapless::Vec;
use nanofish::{
    DefaultEmbassyHttpServer, HttpHandler, HttpRequest, HttpResponse, ResponseBody, StatusCode,
};
use static_cell::StaticCell;

use crate::time::{TIME_UPDATE, TimeUpdate};

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn netrunner_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
pub async fn net_task(_spawner: Spawner, pwr: Output<'static>, spi: PioSpi<'static, PIO0, 0>) {
    let fw = aligned_bytes!("./43439A0.bin");
    let clm = aligned_bytes!("./43439A0_clm.bin");
    let nvram = aligned_bytes!("./nvram_rp2040.bin");

    let mut rng = RoscRng;

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw, nvram).await;
    _spawner.spawn(cyw43_task(runner).unwrap());

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // Use a link-local address for communication without DHCP server
    let config = Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(embassy_net::Ipv4Address::new(192, 168, 88, 1), 16),
        dns_servers: heapless::Vec::new(),
        gateway: None,
    });
    let seed = rng.next_u64();

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        net_device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    );
    _spawner.spawn(netrunner_task(runner).unwrap());
    control.start_ap_wpa2("Azabu Mai", "maimaipolka", 5).await;

    let mut server = DefaultEmbassyHttpServer::new(80); // Listen on port 80
    let handler = MyHandler;

    // This runs forever, handling requests
    server.serve(stack, handler).await;
}

// Create a simple request handler
struct MyHandler;

impl HttpHandler for MyHandler {
    async fn handle_request(
        &mut self,
        request: &HttpRequest<'_>,
    ) -> Result<HttpResponse<'_>, nanofish::Error> {
        match request.path {
            "/" => Ok(HttpResponse {
                status_code: StatusCode::Ok,
                headers: Vec::new(),
                body: ResponseBody::Text(include_str!("./index.html")),
            }),
            "/api/time" => match request.method {
                nanofish::HttpMethod::POST => {
                    // Buat string dari request body
                    let mut vec = Vec::<u8, 14>::new();
                    vec.extend_from_slice(request.body).unwrap();
                    let body = heapless::String::<14>::from_utf8(vec).unwrap();

                    let timestamp: u64 = body.parse().unwrap();
                    TIME_UPDATE.signal(TimeUpdate { timestamp });
                    Ok(HttpResponse {
                        status_code: StatusCode::Ok,
                        headers: Vec::new(),
                        body: ResponseBody::Text("{\"status\":\"ok\"}"),
                    })
                }
                _ => Ok(HttpResponse {
                    status_code: StatusCode::MethodNotAllowed,
                    headers: Vec::new(),
                    body: ResponseBody::Text("Not Found"),
                }),
            },
            "/api/status" => Ok(HttpResponse {
                status_code: StatusCode::Ok,
                headers: Vec::new(),
                body: ResponseBody::Text("{\"status\":\"ok\"}"),
            }),
            _ => Ok(HttpResponse {
                status_code: StatusCode::NotFound,
                headers: Vec::new(),
                body: ResponseBody::Text("Not Found"),
            }),
        }
    }
}
