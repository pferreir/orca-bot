use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::info;
use uxn::{Backend, Uxn, UxnRam};
use varvara::{Output, Varvara};
use zerocopy::IntoBytes;

pub struct VMWrapper<'t> {
    native: bool,
    args: &'t Vec<String>,
    screen_dir: PathBuf,
    audio_file: PathBuf,
}

impl<'t> VMWrapper<'t> {
    pub fn new(
        screen_dir: impl AsRef<Path>,
        audio_file: impl AsRef<Path>,
        args: &'t Vec<String>,
        native: bool,
    ) -> Self {
        Self {
            screen_dir: PathBuf::from(screen_dir.as_ref()),
            audio_file: PathBuf::from(audio_file.as_ref()),
            args,
            native,
        }
    }

    pub fn run(
        &self,
        rom_path: impl AsRef<Path>,
        input: impl Iterator<Item = &'t [char]>,
        n_frames: usize,
    ) -> Result<(u16, u16)> {
        let mut f = std::fs::File::open(rom_path.as_ref())
            .with_context(|| format!("failed to open {:?}", rom_path.as_ref()))?;

        let mut rom = vec![];
        f.read_to_end(&mut rom).context("failed to read file")?;

        let mut ram = UxnRam::new();
        let mut vm = Uxn::new(
            &mut ram,
            if self.native {
                #[cfg(not(target_arch = "aarch64"))]
                anyhow::bail!("no native implementation for this arch");

                #[cfg(target_arch = "aarch64")]
                Backend::Native
            } else {
                Backend::Interpreter
            },
        );
        let mut dev = Varvara::new();
        let data = vm.reset(&rom);
        dev.reset(data);

        // Run the reset vector
        let start = std::time::Instant::now();
        vm.run(&mut dev, 0x100);
        info!("startup complete in {:?}", start.elapsed());

        dev.output(&vm).check()?;
        dev.send_args(&mut vm, self.args).check()?;

        for line in input {
            for c in line.iter() {
                dev.char(&mut vm, *c as u8);
                dev.pressed(&mut vm, varvara::Key::Right, false);
                dev.released(&mut vm, varvara::Key::Right);
            }
            dev.pressed(&mut vm, varvara::Key::Down, false);
            dev.released(&mut vm, varvara::Key::Down);

            for _n in 0..line.len() {
                dev.pressed(&mut vm, varvara::Key::Left, false);
                dev.released(&mut vm, varvara::Key::Left);
            }
        }

        let streams = dev.audio_streams();
        let mut audio_tmp = [0f32; 1470];

        let mut audio_f =
            File::create(self.audio_file.clone()).context("Failed to open output audio file")?;

        for frame_n in 0..n_frames {
            let mut audio_mixdown = [0f32; 1470];

            dev.audio(&mut vm);
            dev.redraw(&mut vm);

            for stream in streams.iter() {
                stream.lock().unwrap().next(&mut audio_tmp);
                for (n, v) in audio_mixdown.iter_mut().enumerate() {
                    *v += audio_tmp[n];
                }
            }

            audio_f
                .write(audio_mixdown.as_bytes())
                .context("Can't write to file")?;
            audio_f.flush()?;

            let out = dev.output(&vm);
            let Output { frame, .. } = out;
            out.check()?;

            let mut f = File::create(self.screen_dir.join(format!("out_{frame_n:05}.rgba")))
                .context("Can't open image output file")?;
            f.write(frame).context("Can't write to file")?;
            f.flush()?;
        }

        let Output { size, .. } = dev.output(&vm);

        Ok(size)
    }
}
