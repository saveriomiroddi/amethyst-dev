use std::{
    io::Write,
    time::{Duration, Instant},
};

use log::info;
use std::{fs::File, io::LineWriter};

const ZERO_DURATION: Duration = Duration::from_millis(0);

// The file is stored in the system temp directory (printed at the beginning of the profiling).
//
const SUBDIRECTORY: &str = "framelimiter_data";
const FILE_HEADER: &str =
    "frame_duration_ms,frame_work_ms,expected_sleep_ms,actual_sleep_ms,skew_ms";

//
#[derive(Debug)]
pub struct FrameLimiterProfiler {
    frame_duration: Duration,
    previous_frame_end: Instant,
    sleep_start: Instant,
    writer: LineWriter<File>,
}

impl FrameLimiterProfiler {
    pub fn new(fps: u32) -> Self {
        // Phony values; set on each cycle.
        //
        let previous_frame_end = Instant::now();
        let sleep_start = Instant::now();

        let output_file = Self::prepare_output_file();
        let writer = LineWriter::new(output_file);

        let mut profiler = Self {
            frame_duration: Duration::from_secs(1) / fps,
            previous_frame_end,
            sleep_start,
            writer,
        };

        profiler.write_header();

        profiler
    }

    pub fn set_frame_duration(&mut self, fps: u32) {
        self.frame_duration = Duration::from_secs(1) / fps;
    }

    // Using a closure as opposed to start/end complicates the design, due to the BCK.
    //
    pub fn mark_sleep_start(&mut self, previous_frame_end: Instant) {
        self.previous_frame_end = previous_frame_end;
        self.sleep_start = Instant::now();
    }

    pub fn compute_and_store_frame_timings(&mut self) {
        let sleep_end = Instant::now();

        let frame_work = self.sleep_start - self.previous_frame_end;
        let expected_sleep = self
            .frame_duration
            .checked_sub(frame_work)
            .unwrap_or(ZERO_DURATION);
        let actual_sleep = sleep_end - self.sleep_start;
        let skew = (actual_sleep.as_micros() - expected_sleep.as_micros()) as f64 / 1000.0;

        if expected_sleep < ZERO_DURATION {}

        self.write_frame_timings(frame_work, expected_sleep, actual_sleep, skew);
    }

    fn prepare_output_file() -> File {
        let current_exe = std::env::current_exe().unwrap();

        info!("Current exe: {:?}", current_exe);

        let subdirectory = std::env::current_dir().unwrap().join(SUBDIRECTORY);

        std::fs::create_dir_all(&subdirectory).unwrap();

        let base_filename = format!(
            "{}_{}.csv",
            current_exe
                .file_stem()
                .unwrap()
                .to_owned()
                .into_string()
                .unwrap(),
            std::env::consts::OS
        );

        let full_filename = subdirectory.join(base_filename);

        info!("Profiling started! Data file: {:?}", full_filename);

        File::create(full_filename).unwrap()
    }

    fn write_header(&mut self) {
        let row = format!("{}\n", FILE_HEADER);

        self.writer.write_all(row.as_bytes()).unwrap();
    }

    fn write_frame_timings(
        &mut self,
        frame_work: Duration,
        expected_sleep: Duration,
        actual_sleep: Duration,
        skew: f64,
    ) {
        // Watch out the units!

        let row = format!(
            "{:.2},{:.2},{:.3},{:.3},{:.3}\n",
            self.frame_duration.as_micros() as f64 / 1000.0,
            frame_work.as_micros() as f64 / 1000.0,
            expected_sleep.as_micros() as f64 / 1000.0,
            actual_sleep.as_micros() as f64 / 1000.0,
            skew, // millis
        );

        self.writer.write_all(row.as_bytes()).unwrap();
    }
}
