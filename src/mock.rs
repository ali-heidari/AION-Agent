use anyhow::Result;
use core::f32;
use csv::{Reader, StringRecord, StringRecordIter, StringRecordsIter};
use std::fs::File;
use std::io;
use std::ops::Div;
use std::path::Path;
use sysinfo::System;
use csv::Writer;
const MEMORY_THRESHOLD: f32 = 0.8;

#[derive(Clone, Debug, PartialEq)]
pub enum DatasetMode {
    SystemMetrics,
    Inputs,
    Generative,
}

#[derive(Debug, Clone)]
pub struct SyntheticState {
    pub cpu: f32,
    pub mem: f32,
    pub swap: f32,
    pub disk: f32,
    pub throughput: f32,
    pub latency: f32,
    pub direction: f32, // +1 for ramp-up, -1 for ramp-down
    pub count: u32,
    records: Vec<Vec<f32>>,
    mode: DatasetMode,
}

impl SyntheticState {
    pub fn new(mode: DatasetMode) -> Self {
        Self {
            cpu: 0.1,
            mem: 0.1,
            swap: 0.05,
            disk: 0.1,
            throughput: 0.95,
            latency: 0.1,
            direction: 1.0,
            count: 0,
            records: vec![],
            mode: mode,
        }
    }

    pub fn from_csv_dataset(&mut self) -> Result<()> {
        let file_path = Path::new("vmCloud_data.csv");
        let mut rdr = csv::Reader::from_path(file_path)?;
        let max = 1.0; //if rng.gen_bool(0.04) {1.0} else {0.4};
        for result in rdr.records() {
            let record = result?;
            if record.get(2).unwrap().is_empty() || record.get(3).unwrap().is_empty() {
                continue;
            }
            let max = 1.0; //if rng.gen_bool(0.04) {1.0} else {0.4};
            let features = vec![
                record
                    .get(2)
                    .unwrap()
                    .parse::<f32>()
                    .unwrap()
                    .div(100.0)
                    .clamp(0.05, max),
                record
                    .get(3)
                    .unwrap()
                    .parse::<f32>()
                    .unwrap()
                    .div(100.0)
                    .clamp(0.05, max),
            ];
            self.records.push(features);
        }
        println!("Dataset has been loaded!");
        Ok(())
    }

    fn generate_by_system_metrics(&mut self) -> Vec<f32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        self.count += 1;

        let mut sys = System::new_all();
        sys.refresh_all();
        self.cpu = sys.global_cpu_usage() / 100.0;

        self.mem = sys.used_memory() as f32 / sys.total_memory() as f32;

        self.swap = sys.used_swap() as f32 / sys.total_swap() as f32;

        let alpha = 0.3; // CPU contribution to disk
        let beta = 0.4; // Memory contribution to disk
        self.disk =
            (alpha * self.cpu + beta * self.mem + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);

        // Throughput inversely related to load
        let gamma = 0.5; // Impact of CPU/memory on throughput
        self.throughput = (1.0 - (self.cpu + self.mem) / 2.0).clamp(0.0, 1.0);
        self.throughput = (self.throughput / 1.0) * (1.0 - gamma * f32::max(self.cpu, self.mem));

        // Latency grows with load
        let delta = 0.5; // Impact of CPU/memory on latency
        self.latency = (self.cpu * 0.6 + self.mem * 0.5).clamp(0.0, 1.0);
        self.latency = f32::min(
            1.0,
            (self.latency / 1.0) * (1.0 + delta * f32::max(self.cpu, self.mem)),
        );



    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("output.csv").unwrap();
    let mut wtr =csv::WriterBuilder::new().from_writer(file);

if self.count == 1 {
    let record = vec!["cpu".to_string(), "memory".to_string(), "swap".to_string(), "disk".to_string(), "throughput".to_string(), "latency".to_string()];
        wtr.write_record(&record);
}

  let record=vec![
            self.cpu.to_string(),
            self.mem.to_string(),
            self.swap.to_string(),
            self.disk.to_string(),
            self.throughput.to_string(),
            self.latency.to_string(),
        ];

        wtr.write_record(&record);
        wtr.flush();

        vec![
            self.cpu,
            self.mem,
            self.swap,
            self.disk,
            self.throughput,
            self.latency,
        ]
    }

    fn generate_by_inputs(&mut self, cpu: f32, mem: f32, lowest_state: u32) -> Vec<f32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut range = (0.01, 1.0);

        if self.count > 200 {
            let state = match lowest_state {
                0 => ((0.65, 1.0), 0.65),
                1 => ((0.35, 0.7), 0.35),
                2 => ((0.01, 0.35), 0.01),
                _ => ((0.01, 1.0), 0.0),
            };
            range = state.0;
            if state.1 > 0.0 {
                self.cpu = state.1;
                self.mem = state.1;
            }
            self.count = 0;
        }

        if rng.gen_bool(0.5) {
            self.direction *= -1.0;
        }

        let record: Vec<f32> = self.records[self.count as usize].clone();

        self.cpu = record[0];

        self.mem = record[1];

        self.swap = f32::max(0.0, self.mem + rng.gen_range(0.0..0.2) - MEMORY_THRESHOLD);

        let alpha = 0.3; // CPU contribution to disk
        let beta = 0.4; // Memory contribution to disk
        self.disk =
            (alpha * self.cpu + beta * self.mem + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);

        // Throughput inversely related to load
        let gamma = 0.5; // Impact of CPU/memory on throughput
        self.throughput = (1.0 - (self.cpu + self.mem) / 2.0).clamp(0.0, 1.0);
        self.throughput = (self.throughput / 1.0) * (1.0 - gamma * f32::max(self.cpu, self.mem));

        // Latency grows with load
        let delta = 0.5; // Impact of CPU/memory on latency
        self.latency = (self.cpu * 0.6 + self.mem * 0.5).clamp(0.0, 1.0);
        self.latency = f32::min(
            1.0,
            (self.latency / 1.0) * (1.0 + delta * f32::max(self.cpu, self.mem)),
        );

        if (self.cpu == range.0 && self.mem == range.0)
            || (self.cpu == range.1 && self.mem == range.1)
        {
            self.direction *= -1.0;
        }

        self.count += 1;

        vec![
            self.cpu,
            self.mem,
            self.swap,
            self.disk,
            self.throughput,
            self.latency,
        ]
    }

    fn generate_by_random(&mut self, lowest_state: u32) -> Vec<f32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut range = (0.01, 1.0);

        if self.count > 200 {
            let state = match lowest_state {
                0 => ((0.65, 1.0), 0.65),
                1 => ((0.35, 0.7), 0.35),
                2 => ((0.01, 0.35), 0.01),
                _ => ((0.01, 1.0), 0.0),
            };
            range = state.0;
            if state.1 > 0.0 {
                self.cpu = state.1;
                self.mem = state.1;
            }
            self.count = 0;
        }

        self.count += 1;

        if rng.gen_bool(0.5) {
            self.direction *= -1.0;
        }

        self.cpu = (self.cpu + self.direction * rng.gen_range(0.01..0.03)).clamp(range.0, range.1);

        self.mem = (self.mem + self.direction * rng.gen_range(0.005..0.01)).clamp(range.0, range.1);

        self.swap = f32::max(0.0, self.mem + rng.gen_range(0.0..0.2) - MEMORY_THRESHOLD);

        let alpha = 0.3; // CPU contribution to disk
        let beta = 0.4; // Memory contribution to disk
        self.disk =
            (alpha * self.cpu + beta * self.mem + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);

        // Throughput inversely related to load
        let gamma = 0.5; // Impact of CPU/memory on throughput
        self.throughput = (1.0 - (self.cpu + self.mem) / 2.0).clamp(0.0, 1.0);
        self.throughput = (self.throughput / 1.0) * (1.0 - gamma * f32::max(self.cpu, self.mem));

        // Latency grows with load
        let delta = 0.5; // Impact of CPU/memory on latency
        self.latency = (self.cpu * 0.6 + self.mem * 0.5).clamp(0.0, 1.0);
        self.latency = f32::min(
            1.0,
            (self.latency / 1.0) * (1.0 + delta * f32::max(self.cpu, self.mem)),
        );

        if (self.cpu == range.0 && self.mem == range.0)
            || (self.cpu == range.1 && self.mem == range.1)
        {
            self.direction *= -1.0;
        }

        vec![
            self.cpu,
            self.mem,
            self.swap,
            self.disk,
            self.throughput,
            self.latency,
        ]
    }

    pub fn next(&mut self, cpu: f32, mem: f32, lowest_state: u32) -> Vec<f32> {
        match self.mode {
            DatasetMode::Generative => self.generate_by_random(lowest_state),
            DatasetMode::Inputs => self.generate_by_inputs(cpu, mem, lowest_state),
            DatasetMode::SystemMetrics => self.generate_by_system_metrics(),
        }
    }
}
