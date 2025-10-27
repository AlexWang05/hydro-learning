use hydro_deploy::Deployment;

#[tokio::main]
async fn main() {
    let mut d = Deployment::new();

    let flow   = hydro_lang::compile::builder::FlowBuilder::new();
    let client = flow.process::<hydro_template::two_pc::Client>();
    let coord  = flow.process::<hydro_template::two_pc::Coord>();
    let parts  = flow.cluster::<hydro_template::two_pc::Participant>();

    hydro_template::two_pc::two_pc(&client, &coord, &parts);

    let _nodes = flow
        .with_process(&client, d.Localhost())
        .with_process(&coord,  d.Localhost())
        .with_cluster(&parts,  vec![d.Localhost(); 2]) // num participants
        .deploy(&mut d);

    d.run_ctrl_c().await.unwrap();
}
