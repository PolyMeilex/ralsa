use std::os::unix::prelude::AsRawFd;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = udev::MonitorBuilder::new()
        .unwrap()
        .match_subsystem("sound")
        .unwrap()
        .listen()
        .unwrap();

    let monitor_fd = monitor.as_raw_fd();

    let mut fds = vec![nix::poll::PollFd::new(
        monitor_fd,
        nix::poll::PollFlags::POLLIN,
    )];

    let mut enumerator = udev::Enumerator::new()?;
    enumerator.match_subsystem("sound")?;

    // let seq = Seq::open(None, None, true)?;
    // println!("Alsa:");
    // for client in ClientIter::new(&seq) {
    //     let card = client.get_card();
    //     if let Some(card_id) = card {
    //         println!();
    //         println!("Card: {:?}", card_id);
    //         println!("Card Name: {:?}", client.get_name());
    //     }
    // }

    println!("Udev:");
    println!();

    for device in enumerator.scan_devices()? {
        if let Some(card_id) = get_card_id(&device) {
            println!();
            println!("Card: {:?}", card_id);
            // println!("{:#?}", device);

            // let vendor = device.property_value("ID_VENDOR");
            let bus = device.property_value("ID_BUS");
            let path = device.property_value("ID_PATH");
            let model = device.property_value("ID_MODEL_ID");
            let vendor = device.property_value("ID_VENDOR_ENC");
            let vendor_id = device.property_value("ID_VENDOR_ID");
            let usb_interface_num = device.property_value("ID_USB_INTERFACE_NUM");
            let serial = device.property_value("ID_SERIAL_SHORT");

            dbg!(
                bus,
                path,
                model,
                vendor,
                vendor_id,
                usb_interface_num,
                serial
            );

            // println!("  [properties]");
            // for property in device.properties() {
            //     println!("    - {:?} {:?}", property.name(), property.value());
            // }

            // println!("  [attributes]");
            // for attribute in device.attributes() {
            //     println!("    - {:?} {:?}", attribute.name(), attribute.value());
            // }
        }
    }

    loop {
        nix::poll::poll(&mut fds, -1).unwrap();

        if fds[0].revents() == Some(nix::poll::PollFlags::POLLIN) {
            let mut changed = false;

            for event in monitor.by_ref() {
                let init = event.property_value("SOUND_INITIALIZED");

                if init.is_some() {
                    let device = event.device();
                    let is_card = device.syspath().to_string_lossy().contains("card");

                    if is_card {
                        dbg!(device.sysnum());

                        changed = true;
                    } else {
                        println!("This is not a card: {:#?}", device);
                    }
                }
            }

            if changed {
                //
            }
        }

        dbg!("Event");
    }
}

fn get_card_id(device: &udev::Device) -> Option<usize> {
    let is_card = device.syspath().to_string_lossy().contains("card");

    if is_card {
        device.sysnum()
    } else {
        None
    }
}
