import multiprocessing
import threading
import time
import gc
import os

ram_up_time = 3  # Time to ramp up resource usage in seconds
hold_time = 30    # Time to hold high resource usage in seconds
release_time = 1.5 # Time to release resource usage in seconds
low_hold_time = 30 # Time to hold low resource usage in seconds

import time

def busy_loop_with_target(target_percent=70.0, duration_ms=1000):
    """
    Consumes CPU for a target percentage of the time.
    
    Args:
        target_percent (float): The target CPU usage as a percentage (e.g., 50.0).
        duration_ms (int): The duration of each busy/sleep cycle in milliseconds.
    """
    if not 0.0 <= target_percent <= 100.0:
        raise ValueError("Target CPU percentage must be between 0 and 100.")
        
    # Calculate the split between busy and sleep time
    busy_ms = duration_ms * (target_percent / 100.0)
    sleep_ms = duration_ms - busy_ms
    
    start_time = time.time() * 1000 # Convert to milliseconds

    while True:
        # Busy part: Consume CPU by running a tight loop
        current_time = time.time() * 1000
        while (time.time() * 1000) - current_time < busy_ms:
            pass
        
        # Sleep part: Give the CPU a break
        if sleep_ms > 0:
            time.sleep(sleep_ms / 1000.0)



def busy_loop():
    while True:
        pass  # Infinite loop to consume CPU on one core

def cpu_load():
    num_cores = multiprocessing.cpu_count()
    while True:
        print("CPU: Ramping up...")
        processes = []
        for i in range(num_cores):
            p = multiprocessing.Process(target=busy_loop_with_target)
            p.start()
            processes.append(p)
            time.sleep(ram_up_time*3)  # Slower CPU ramp-up (2s per core)
        
        print("CPU: Holding high usage...")
        time.sleep(hold_time)  # Hold high CPU for 20s

        print("CPU: Slowly freeing up...")
        for p in processes:
            p.terminate()
            p.join()
            time.sleep(release_time*2)  # Slower CPU release (3s per core)

        print("CPU: Holding low usage...") 
        time.sleep(low_hold_time*2)  # Hold low CPU for 15s

def memory_load():
    chunk_size_mb = 100  # Memory chunk size in MB
    num_chunks = 70 # Number of memory chunks
    while True:
        print("Memory: Ramping up...")
        mem_chunks = []
        for i in range(num_chunks):
            mem_chunks.append(bytearray(chunk_size_mb * 1024 * 1024))
            time.sleep(ram_up_time/6)  # Faster memory ramp-up (1.5s per chunk)
        
        print("Memory: Holding high usage...")
        time.sleep(hold_time)  # Hold high memory for 25s

        print("Memory: Slowly freeing up...")
        for i in range(len(mem_chunks)):
            mem_chunks.pop()
            gc.collect()
            time.sleep(release_time)  # Faster memory release (2.5s per chunk)

        print("Memory: Holding low usage...")
        time.sleep(low_hold_time)  # Hold low memory for 10s

def disk_load():
    disk_file = "temp_disk_file.dat"
    max_disk_size_mb = 50 * 1000  # Max disk usage in MB
    disk_chunk_mb = 100  # Disk growth per step in MB
    while True:
        # Ensure clean start
        if os.path.exists(disk_file):
            os.remove(disk_file)

        print("Disk: Ramping up...")
        with open(disk_file, "ab") as f:
            for i in range(0, max_disk_size_mb, disk_chunk_mb):
                f.write(bytearray(disk_chunk_mb * 1024 * 1024))
                f.flush()
                os.fsync(f.fileno())
                time.sleep(ram_up_time)  # Slower disk ramp-up (3s per chunk)
                print(f"Disk: Usage at {(i + disk_chunk_mb)} MB")

        print("Disk: Holding high usage...")
        time.sleep(30)  # Hold high disk for 30s

        print("Disk: Slowly freeing up...")
        with open(disk_file, "ab") as f:
            for size in range(max_disk_size_mb - disk_chunk_mb, -1, -disk_chunk_mb):
                f.truncate(size * 1024 * 1024)
                f.flush()
                os.fsync(f.fileno())
                time.sleep(release_time)  # Slower disk release (4s per chunk)
                print(f"Disk: Usage reduced to {size} MB")

        # Clean up file
        if os.path.exists(disk_file):
            os.remove(disk_file)

        print("Disk: Holding low usage...")
        time.sleep(low_hold_time)  # Hold low disk for 20s

if __name__ == "__main__":
    # Start each load in a separate thread
    cpu_thread = threading.Thread(target=cpu_load, daemon=True)
    memory_thread = threading.Thread(target=memory_load, daemon=True)
    # disk_thread = threading.Thread(target=disk_load, daemon=True)

    cpu_thread.start()
    memory_thread.start()

    disk_file = "temp_disk_file.dat"
    # Ensure clean start
    if os.path.exists(disk_file):
        os.remove(disk_file)
    # disk_thread.start()

    try:
        # Keep the main thread running indefinitely
        while True:
            time.sleep(100)
    except KeyboardInterrupt:
        print("Shutting down...")
        # Clean up disk file on exit
        if os.path.exists("temp_disk_file.dat"):
            os.remove("temp_disk_file.dat")
