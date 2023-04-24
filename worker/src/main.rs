use tonic::{transport::Server, Request, Response, Status};

use local_ip_address::local_ip;
use pcap::Device;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use tokio::time::sleep;

use bookstore::bookstore_server::{Bookstore, BookstoreServer};
use bookstore::{GetBookRequest, GetBookResponse};

mod bookstore {
    include!("bookstore.rs");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("greeter_descriptor");
}

#[derive(Default)]
pub struct BookStoreImpl {}

#[tonic::async_trait]
impl Bookstore for BookStoreImpl {
    async fn get_book(
        &self,
        request: Request<GetBookRequest>,
    ) -> Result<Response<GetBookResponse>, Status> {
        println!("Request from {:?}", request.remote_addr());

        let response = GetBookResponse {
            id: request.into_inner().id,
            author: "Peter".to_owned(),
            name: "Zero to One".to_owned(),
            year: 2014,
        };
        Ok(Response::new(response))
    }
}

async fn capture_outbound_packets(
    local_ip_address: String,
    sent_bytes: Arc<AtomicU64>,
) -> Result<(), pcap::Error> {
    let device = Device::lookup()?;
    let mut capture = device.unwrap().open()?;

    capture.filter(&format!("src host {local_ip_address}"), true)?; // Capture only outbound packets

    loop {
        let packet = capture.next_packet()?;
        sent_bytes.fetch_add(packet.header.len as u64, Ordering::SeqCst);
    }
}

// Calculates the average outbound bandwidth rate in bytes per second and returns it
async fn calculate_average_outbound_bandwidth(
    average_outbound_bandwidth_per_second: Arc<AtomicU64>,
    sent_bytes: Arc<AtomicU64>,
) {
    // The index is used to calculate the average bandwidth rate (how many times the loop has run)
    let mut index = 0;

    loop {
        // Sent bandwidth before the sleep
        let initial_sent_bytes = sent_bytes.load(Ordering::SeqCst);

        // Sleep for 30 seconds
        sleep(Duration::from_secs(2)).await;

        // Sent bandwidth after the sleep
        let current_sent_bytes = sent_bytes.load(Ordering::SeqCst);

        index += 1;

        let outbound_transferred_during_sleep_per_sec =
            (current_sent_bytes - initial_sent_bytes) / 2;

        average_outbound_bandwidth_per_second.store(
            (average_outbound_bandwidth_per_second.load(Ordering::SeqCst)
                + outbound_transferred_during_sleep_per_sec)
                / index,
            Ordering::SeqCst,
        );

        println!(
            "Average outbound bandwidth rate: {} bytes/s",
            average_outbound_bandwidth_per_second.load(Ordering::SeqCst)
        );
    }
}

async fn bandwidth_listener() -> Result<(), Box<dyn std::error::Error>> {
    let sent_bytes = Arc::new(AtomicU64::new(0));
    let average_outbound_bandwidth_per_second = Arc::new(AtomicU64::new(0));

    let capture_task = task::spawn(capture_outbound_packets(
        local_ip().unwrap().to_string(),
        sent_bytes.clone(),
    ));

    let poll_task = task::spawn(calculate_average_outbound_bandwidth(
        average_outbound_bandwidth_per_second,
        sent_bytes,
    ));

    let _ = tokio::try_join!(capture_task, poll_task)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    bandwidth_listener().await;


    let addr = "[::1]:50051".parse().unwrap();
    let bookstore = BookStoreImpl::default();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(bookstore::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    println!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(BookstoreServer::new(bookstore))
        .add_service(reflection_service) // Add this
        .serve(addr)
        .await?;

    Ok(())
}
