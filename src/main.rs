pub mod danmu;

fn main() {
    let matches = clap::App::new("douyu-danmu")
        .version("0.1.0")
        .author("divinerapier <poriter.coco@gmail.com>")
        .arg(
            clap::Arg::with_name("room")
                .short("r")
                .long("room")
                .value_name("room_id")
                .help("Sets a custom room id")
                .takes_value(true),
        )
        .get_matches();
    let room_id = matches.value_of("room").unwrap();
    let room_id = room_id.parse().unwrap();

    let mut danmu = danmu::Danmu::new("openbarrage.douyutv.com:8601");
    danmu.login(room_id);
    danmu.join_group(room_id);
    danmu.keep_alive();
    danmu.run();
}
