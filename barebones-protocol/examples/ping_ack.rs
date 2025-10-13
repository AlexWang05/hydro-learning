use hydro_deploy::Deployment;

#[tokio::main]
async fn main() {
    let mut d = Deployment::new();

    let flow   = hydro_lang::compile::builder::FlowBuilder::new();
    let client = flow.process::<hydro_template::ping_ack::Client>();
    let coord  = flow.process::<hydro_template::ping_ack::Coord>();
    let parts  = flow.cluster::<hydro_template::ping_ack::Participant>();

    hydro_template::ping_ack::ping_ack(&client, &coord, &parts);


    let _nodes = flow
        .with_process(&client, d.Localhost())
        .with_process(&coord,  d.Localhost())
        .with_cluster(&parts,  vec![d.Localhost(); 2])
        .deploy(&mut d);

    d.run_ctrl_c().await.unwrap();
}
