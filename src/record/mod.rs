pub mod fps;
pub mod settings;

use crate::encode::gif::Frame;
use crate::encode::Image;
use crate::record::fps::{FpsClock, TimeUnit};
use crate::record::settings::RecordSettings;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/* Asynchronous recording result */
#[derive(Debug)]
pub struct RecordResult<T> {
	sender: mpsc::Sender<()>,
	thread: thread::JoinHandle<T>,
}

impl<T> RecordResult<T> {
	/**
	 * Create a new RecordResult object.
	 *
	 * @param  sender
	 * @param  thread
	 * @return RecordResult
	 */
	pub fn new(sender: mpsc::Sender<()>, thread: thread::JoinHandle<T>) -> Self {
		Self { sender, thread }
	}

	/**
	 * Stop the thread and retrieve values.
	 *
	 * @return Option
	 */
	pub fn get(self) -> Option<thread::Result<T>> {
		if self.sender.send(()).is_ok() {
			Some(self.thread.join())
		} else {
			None
		}
	}
}

/* Recorder with FPS clock and channel */
pub struct Recorder {
	clock: FpsClock,
	channel: (mpsc::Sender<()>, mpsc::Receiver<()>),
	get_image: &'static (dyn Fn() -> Option<Image> + Sync + Send),
}

impl Recorder {
	/**
	 * Create a new Recorder object.
	 *
	 * @param  settings
	 * @param  get_image (Fn)
	 * @return Recorder
	 */
	pub fn new(
		settings: RecordSettings,
		get_image: impl Fn() -> Option<Image> + Sync + Send + 'static,
	) -> Self {
		Self {
			clock: FpsClock::new(settings.fps),
			channel: mpsc::channel(),
			get_image: Box::leak(Box::new(get_image)),
		}
	}

	/**
	 * Get a frame from calling the image function.
	 *
	 * @return Frame
	 */
	fn get_frame(&mut self) -> Frame {
		match (self.get_image)() {
			Some(image) => Frame::new(
				image,
				(self.clock.get_fps(TimeUnit::Millisecond) / 10.) as u16,
			),
			None => panic!("Failed to get the image"),
		}
	}

	/**
	 * Record frames synchronously with blocking the current thread.
	 *
	 * @return Vector of Frame
	 */
	pub fn record_sync(&mut self) -> Vec<Frame> {
		let mut frames = Vec::new();
		let recording = Arc::new(AtomicBool::new(true));
		let rec_state = recording.clone();
		ctrlc::set_handler(move || {
			rec_state.store(false, Ordering::SeqCst);
		})
		.expect("Failed to set the signal handler");
		while recording.load(Ordering::SeqCst) {
			self.clock.tick();
			frames.push(self.get_frame());
		}
		frames
	}

	/**
	 * Record frames asynchronously and without blocking.
	 *
	 * @return RecordResult
	 */
	pub fn record_async(mut self) -> RecordResult<Vec<Frame>> {
		let mut frames = Vec::new();
		RecordResult::new(
			self.channel.0.clone(),
			thread::spawn(move || {
				thread::sleep(Duration::from_millis(
					self.clock.get_fps(TimeUnit::Millisecond) as u64,
				));
				while self.channel.1.try_recv().is_err() {
					self.clock.tick();
					frames.push(self.get_frame())
				}
				frames
			}),
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::encode::Geometry;
	use std::thread;
	use std::time::Duration;
	#[test]
	fn test_record_mod() {
		let recorder = Recorder::new(RecordSettings::default(), move || {
			Some(Image::new(
				vec![0, 0, 0, 255, 255, 255],
				Geometry::new(0, 0, 1, 1),
			))
		});
		let record = recorder.record_async();
		thread::sleep(Duration::from_millis(200));
		record.finish().unwrap();
		assert!(record.thread.join().unwrap().len() > 0);
	}
}
