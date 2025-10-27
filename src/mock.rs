use core::f32;
use sysinfo::System;

const MEMORY_THRESHOLD: f32 = 0.8;

pub enum Mode{
    SystemMetrics,
    Inputs,
    Generative
}

#[derive(Debug, Clone)]
pub struct SyntheticState {
    pub cpu: f32,
    pub mem: f32,
    pub swap: f32,
    pub disk: f32,
    pub throughput: f32,
    pub latency: f32,
    pub direction: f32,// +1 for ramp-up, -1 for ramp-down
    pub state: u32, 
    pub count: u32
}

impl SyntheticState {
    pub fn new() -> Self {
        Self {
            cpu: 0.1,
            mem: 0.1,
            swap: 0.05,
            disk: 0.1,
            throughput: 0.95,
            latency: 0.1,
            direction: 1.0,
            state:0,
            count:1
        }
    }

    pub fn next(&mut self,mode:Mode, cpu:f32,mem:f32,lowest_state:u32) -> Vec<f32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

let mut range = (0.01,1.0); 

if self.count>200 {
    
   let state= match lowest_state {
    0 => ((0.65,1.0), 0.65),
    1 => ((0.35,0.7), 0.35),
    2 => ((0.01,0.35), 0.01),
    _ => ((0.01,1.0), 0.0)

};
range=state.0;
if(state.1>0.0){
    self.cpu=state.1;
    self.mem=state.1;}
self.count=0;
}

// range=if rng.gen_bool(0.5){(0.05,1.0)} else {range};
// self.state=lowest_state;
self.count+=1;
        //  println!("lowest_state: {}, range: {:?}",lowest_state,range);


        // Flip direction sometimes
        if rng.gen_bool(0.5) {
            // 2% chance to switch ramp direction
            self.direction *= -1.0;
        } 

        // CPU & memory ramp
        // First we update all information of our `System` struct.
    //    let mut sys = System::new_all();
    //    sys.refresh_all();
        self.cpu = 
        // cpu;
        //    sys.global_cpu_usage() / 100.0; 
        (self.cpu + self.direction * rng.gen_range(0.01..0.03)).clamp(range.0,range.1);
        
        self.mem = 
        // mem;
            // sys.used_memory() as f32 / sys.total_memory() as f32;
         (self.mem + self.direction * rng.gen_range(0.005..0.01)).clamp(range.0, range.1);

        // Swap rises if memory high
        self.swap =  
            // sys.used_swap() as f32 / sys.total_swap() as f32;
            f32::max(0.0, self.mem + rng.gen_range(0.0..0.2) - MEMORY_THRESHOLD);
        //(self.swap * 0.9 + if self.mem > 0.8 { 0.02 } else { -0.01 }).clamp(0.0, 0.3);

        // Disk random variation with light correlation
        let alpha = 0.3; // CPU contribution to disk
        let beta = 0.4; // Memory contribution to disk
        self.disk =
            (alpha * self.cpu + beta * self.mem + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);
        // self.disk = (self.disk * 0.8 + self.cpu * 0.2 + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);

        // Throughput inversely related to load
        let gamma = 0.5; // Impact of CPU/memory on throughput
        self.throughput = (1.0 - (self.cpu + self.mem) / 2.0).clamp(0.0, 1.0);
        self.throughput = (self.throughput / 1.0) * (1.0 - gamma * f32::max(self.cpu, self.mem));

        // Latency grows with load
        let delta = 0.5; // Impact of CPU/memory on latency
        self.latency = (self.cpu * 0.6 + self.mem * 0.5).clamp(0.0, 1.0);
        self.latency = f32::min(
            1.0,
            (self.latency/1.0) * (1.0 + delta * f32::max(self.cpu, self.mem)),
        );

        if (self.cpu == range.0 && self.mem==range.0) ||(self.cpu == range.1 && self.mem==range.1){
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
}
