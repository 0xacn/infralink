pub mod data;
pub mod models;

use tonic::transport::Server;
use tonic::{Request, Response, Status};

use proto_memory::memory_service_server::{MemoryService, MemoryServiceServer};
use proto_memory::MemoryMetadata;

use proto_compute::compute_service_server::{ComputeService, ComputeServiceServer};
use proto_compute::ComputeMetadata;

mod proto_compute {
	include!("compute.rs");
}

mod proto_memory {
	include!("memory.rs");

	pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
		tonic::include_file_descriptor_set!("greeter_descriptor");
}

#[derive(Default)]
pub struct ComputeServiceImpl {}

#[derive(Default)]
pub struct MemoryServiceImpl {}

#[derive(Default)]
pub struct StorageServiceImpl {}

#[derive(Default)]
pub struct NetworkServiceImpl {}

// impl NetworkServiceServer {
// pub fn start() {}
// }

#[tonic::async_trait]
impl ComputeService for ComputeServiceImpl {
	async fn get_compute_metadata(
		&self,
		_request: Request<()>,
	) -> Result<Response<ComputeMetadata>, Status> {
		// Fetch compute metadata
		let metadata = data::compute::compute_usage();

		// Convert the compute metadata into the protobuf struct

		let cpus = metadata
			.cpus
			.unwrap()
			.iter()
			.map(|cpu| {
				proto_compute::Cpu {
					frequency: cpu.frequency.unwrap(),
					load: cpu.load.unwrap(),
				}
			})
			.collect();

		Ok(Response::new(proto_compute::ComputeMetadata {
			num_cores: metadata.num_cores.unwrap(),
			cpus: cpus,
		}))
	}
}

#[tonic::async_trait]
impl MemoryService for MemoryServiceImpl {
	async fn get_memory_metadata(
		&self,
		_request: Request<()>,
	) -> Result<Response<MemoryMetadata>, Status> {
		// Fetch memory metadata
		let metadata = data::mem::memory();

		// Convert the memory metadata into the protobuf struct
		let primary = metadata.clone().primary.unwrap();
		let swap = metadata.clone().swap.unwrap();

		Ok(Response::new(proto_memory::MemoryMetadata {
			primary: Some(proto_memory::Memory {
				total: primary.total.unwrap(),
				used: primary.used.unwrap(),
				free: primary.free.unwrap(),
			}),
			swap: Some(proto_memory::Memory {
				total: swap.total.unwrap(),
				used: swap.used.unwrap(),
				free: swap.free.unwrap(),
			}),
		}))
	}
}

// #[tonic::async_trait]
// impl StorageService for StorageServiceImpl {
// 	async fn get_storage_metadata(
// 		&self,
// 		_request: Request<()>,
// 	) -> Result<Response<StorageMetadata>, Status> {
// 		// Fetch storage metadata
// 		let metadata = data::storage::storage();

// 		// Convert the storage metadata into the protobuf struct
// 		let primary = metadata.clone().primary.unwrap();
// 		let volumes = metadata
// 			.clone()
// 			.volumes
// 			.unwrap()
// 			.iter()
// 			.map(|volume| {
// 				proto_storage::Volume {
// 					total: volume.total.unwrap(),
// 					used: volume.used.unwrap(),
// 					free: volume.free.unwrap(),
// 				}
// 			})
// 			.collect();

// 		Ok(Response::new(proto_memory::MemoryMetadata {
// 			primary: Some(proto_memory::Memory {
// 				total: primary.total.unwrap(),
// 				used: primary.used.unwrap(),
// 				free: primary.free.unwrap(),
// 			}),
// 			volumes,
// 		}))
// 	}
// }

// #[tonic::async_trait]
// impl NetworkService for NetworkServiceImpl {
// 	async fn get_network_metadata(
// 		&self,
// 		_request: Request<()>,
// 	) -> Result<Response<MemoryMetadata>, Status> {
// 		// Fetch storage metadata
// 		let metadata = data::network::network();

// 		// Convert the storage metadata into the protobuf struct
// 		let primary = metadata.clone().primary.unwrap();
// 		let volumes = metadata
// 			.clone()
// 			.volumes
// 			.unwrap()
// 			.iter()
// 			.map(|volume| {
// 				proto_storage::Volume {
// 					total: volume.total.unwrap(),
// 					used: volume.used.unwrap(),
// 					free: volume.free.unwrap(),
// 				}
// 			})
// 			.collect();

// 		Ok(Response::new(proto_memory::MemoryMetadata {
// 			primary: Some(proto_memory::Memory {
// 				total: primary.total.unwrap(),
// 				used: primary.used.unwrap(),
// 				free: primary.free.unwrap(),
// 			}),
// 			volumes,
// 		}))
// 	}
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:50051".parse().unwrap();

	// Initialize the memory, compute, storage, and network measurement services
	let memory_service = MemoryServiceImpl::default();
	let compute_service = ComputeServiceImpl::default();
	// let storage_service = StorageServiceImpl::default();
	// let network_service = NetworkServiceImpl::default();

	// Create the gRPC servers for each service
	let memory_server = MemoryServiceServer::new(memory_service);
	let compute_server = ComputeServiceServer::new(compute_service);
	// let network_service = NetworkServiceServer::new(network_service);
	// let storage_server = StorageServiceServer::new(storage_service);

	let reflection_service = tonic_reflection::server::Builder::configure()
		.register_encoded_file_descriptor_set(proto_memory::FILE_DESCRIPTOR_SET)
		.build()
		.unwrap();

	println!("gRPC server listening on {}", addr);

	Server::builder()
		.add_service(memory_server)
		.add_service(compute_server)
		// .add_service(storage_server)
		// .add_service(network_service)
		.add_service(reflection_service)
		.serve(addr)
		.await?;

	Ok(())
}
