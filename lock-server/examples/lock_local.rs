use hydro_deploy::{Deployment, NetworkHint};
use hydro_lang::compile::prelude::*;
use tokio_util::codec::LinesCodec;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();

    let flow = hydro_lang::compile::builder::FlowBuilder::new();
    let process = flow.process::<hydro_template::LockServer>();
    let external = flow.external::<()>();

    // start lock server via cargo run --example lock_local
    // in another terminal: nc localhost 4000
    // type commands like Alice,true,0 (alice acquires lock 0)

    // Bind server to accept requests on TCP port 4000
    // Input format: "name,acquire,lock_id"
    //   - name: person name (e.g., "Alice")
    //   - acquire: true or false (e.g., "true" or "false")
    //   - lock_id: lock ID number (e.g., "0")
    // Example: "Alice,true,0" (Alice acquires lock 0)
    //          "Alice,false,0" (Alice releases lock 0)
    let (_port, input, output) = process.bind_single_client::<_, _, LinesCodec>(
        &external,
        NetworkHint::TcpPort(Some(4000)),
    );

    // Parse input: "name,acquire,lock_id" -> (String, bool, usize)
    let parsed_input = input.map(q!(|line: String| {
        let parts: Vec<&str> = line.trim().split(',').collect();
        if parts.len() != 3 {
            panic!("Invalid input format. Expected: name,acquire,lock_id");
        }
        let name = parts[0].to_string();
        let acquire = parts[1].parse::<bool>().expect("acquire must be true or false");
        let lock_id = parts[2].parse::<usize>().expect("lock_id must be a number");
        (name, acquire, lock_id)
    }));

    // Run lock server
    let responses = hydro_template::lock_server(parsed_input);

    // Format output: (String, bool, usize) -> "name,success,lock_id"
    let formatted_output = responses.map(q!(|(name, success, lock_id)| {
        format!("{},{},{}", name, success, lock_id)
    }));

    output.complete(formatted_output);

    let _nodes = flow
        .with_process(&process, deployment.Localhost())
        .with_external(&external, deployment.Localhost())
        .deploy(&mut deployment);

    deployment.deploy().await.unwrap();

    println!("=== Lock Server Started ===");
    println!("Connect with: nc localhost 4000");
    println!();
    println!("Input format: name,acquire,lock_id");
    println!("  - name: person name (e.g., Alice)");
    println!("  - acquire: true to acquire, false to release");
    println!("  - lock_id: lock ID number (e.g., 0)");
    println!();
    println!("Examples:");
    println!("  Alice,true,0    # Alice acquires lock 0");
    println!("  Bob,true,0      # Bob tries to acquire lock 0 (will fail if Alice holds it)");
    println!("  Alice,false,0   # Alice releases lock 0");
    println!();
    println!("Output format: name,success,lock_id");
    println!("  - success: true if operation succeeded, false otherwise");
    println!("===========================");

    deployment.start_ctrl_c().await.unwrap();
}
