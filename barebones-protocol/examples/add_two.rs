use hydro_deploy::Deployment;

#[tokio::main]

async fn main() {
    let mut d = Deployment::new();

    // build the dataflow graph
    let flow  = hydro_lang::compile::builder::FlowBuilder::new();
    let solo  = flow.process::<hydro_template::add_two::Solo>();     // create one process
    hydro_template::add_two::add_two(&solo); // install the graph

    // deploy that process to localhost and run
    let _nodes = flow.with_process(&solo, d.Localhost()).deploy(&mut d);

    d.run_ctrl_c().await.unwrap();
}
