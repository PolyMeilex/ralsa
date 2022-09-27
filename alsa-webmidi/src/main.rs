use alsa_webmidi::MIDIAccess;

fn main() {
    let acces = MIDIAccess::new();

    for input in acces.inputs() {
        input.open(|event| {
            dbg!(event.source());
        });
    }

    acces.run();
}
