use gst::prelude::*;
use anyhow::Error;

fn main() -> Result<(), Error>  {
    gst::init()?;
    gst_plugin_hlssink3::plugin_register_static()?;

    let pipeline = gst::parse_launch("videotestsrc num-buffers=2500 ! timecodestamper ! video/x-raw,format=I420,width=1280,height=720,framerate=30/1 ! timeoverlay ! x264enc bframes=0 bitrate=2048 key-int-max=180 option-string=scenecut=0:force-cfr=1 ! video/x-h264,stream-format=avc,alignment=au,profile=main ! h264parse ! tee name=t ! queue ! hlssink3 name=hls-output t. ! queue ! fakesink sync=true").unwrap().downcast::<gst::Pipeline>().unwrap();
    let hls_output = pipeline.by_name("hls-output").unwrap();
    hls_output.set_properties(&[
        ("playlist-type", &"vod"),
        ("location", &"s%05d.ts"),
        ("playlist-location", &"master.m3u8"),
        ("target-duration", &6u32),
    ]);

    pipeline.set_state(gst::State::Playing)?;

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => {
                println!("EOS");
                break;
            }
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null)?;
                eprintln!(
                    "Got error from {}: {} ({})",
                    msg.src()
                        .map(|s| String::from(s.path_string()))
                        .unwrap_or_else(|| "None".into()),
                    err.error(),
                    err.debug().unwrap_or_else(|| "".into()),
                );
                break;
            }
            _ => {},
        }
    }

    pipeline.set_state(gst::State::Null)?;

    Ok(())
}
