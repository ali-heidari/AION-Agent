import multiprocessing
import threading
import time
import gc
import os
import random


def busy_loop_with_target(target_percent=70.0, duration_ms=1000):
    busy_ms = duration_ms * (target_percent / 100.0)
    sleep_ms = duration_ms - busy_ms

    while True:
        current_time = time.time() * 1000
        while (time.time() * 1000) - current_time < busy_ms:
            pass
        if sleep_ms > 0:
            time.sleep(sleep_ms / 1000.0)


def cpu_load():
    num_cores = multiprocessing.cpu_count()
    while True:
        ram_up_time = random.uniform(1, 10)
        hold_time = random.uniform(20, 60)
        release_time = random.uniform(1, 10)
        low_hold_time = random.uniform(20, 60)
        cores_to_use = random.randint(1, num_cores)
        cpu_percent = random.uniform(40.0, 95.0)

        print(f"CPU: Ramping up ({cores_to_use}/{num_cores} cores @ {cpu_percent:.0f}%)...")
        processes = []
        for i in range(cores_to_use):
            p = multiprocessing.Process(
                target=busy_loop_with_target,
                kwargs={"target_percent": cpu_percent},
            )
            p.start()
            processes.append(p)
            time.sleep(ram_up_time * 3)

        print("CPU: Holding high usage...")
        time.sleep(hold_time)

        print("CPU: Slowly freeing up...")
        for p in processes:
            p.terminate()
            p.join()
            time.sleep(release_time * 2)

        print("CPU: Holding low usage...")
        time.sleep(low_hold_time * 2)


def memory_load():
    chunk_size_mb = 5
    while True:
        ram_up_time = random.uniform(1, 10)
        hold_time = random.uniform(20, 60)
        release_time = random.uniform(1, 10)
        low_hold_time = random.uniform(20, 60)
        num_chunks = random.randint(3, 12)

        print(f"Memory: Ramping up ({num_chunks} chunks × {chunk_size_mb} MB)...")
        mem_chunks = []
        for i in range(num_chunks):
            mem_chunks.append(bytearray(chunk_size_mb * 1024 * 1024))
            time.sleep(ram_up_time / 6)

        print("Memory: Holding high usage...")
        time.sleep(hold_time)

        print("Memory: Slowly freeing up...")
        for i in range(len(mem_chunks)):
            mem_chunks.pop()
            gc.collect()
            time.sleep(release_time)

        print("Memory: Holding low usage...")
        time.sleep(low_hold_time)


if __name__ == "__main__":
    # Stagger startup so agents don't begin cycles in lockstep
    time.sleep(random.uniform(0, 30))

    cpu_thread = threading.Thread(target=cpu_load, daemon=True)
    memory_thread = threading.Thread(target=memory_load, daemon=True)

    cpu_thread.start()
    memory_thread.start()

    try:
        while True:
            time.sleep(100)
    except KeyboardInterrupt:
        print("Shutting down...")
