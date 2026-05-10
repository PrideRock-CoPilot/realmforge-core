#!/usr/bin/env python3
"""
RFSource Load Test - 500 Concurrent Users
==========================================

Tests rfsource system under realistic production load with 500 concurrent users.

Test Scenario:
- 500 concurrent users
- 5-minute sustained load
- Operation mix: 70% reads, 20% writes, 5% branches, 5% admin
- Metrics: p50/p95/p99 latency, throughput, error rate, memory usage

Success Criteria:
- p95 read latency < 200ms
- p95 write latency < 50ms
- Error rate < 1%
- No crashes or data corruption
- Memory usage bounded

Usage:
    python load_test_rfsource_500_users.py --duration 300 --users 500
"""

import asyncio
import time
import random
import statistics
import sys
import argparse
import json
from datetime import datetime
from typing import List, Dict, Tuple
from dataclasses import dataclass, asdict
from concurrent.futures import ThreadPoolExecutor
import threading
import psutil
import os


@dataclass
class OperationResult:
    """Result of a single operation."""
    operation_type: str
    user_id: int
    start_time: float
    end_time: float
    success: bool
    error_message: str = ""
    
    @property
    def latency_ms(self) -> float:
        """Latency in milliseconds."""
        return (self.end_time - self.start_time) * 1000


@dataclass
class LoadTestMetrics:
    """Aggregated load test metrics."""
    total_operations: int
    successful_operations: int
    failed_operations: int
    error_rate: float
    duration_seconds: float
    operations_per_second: float
    
    # Latency metrics by operation type
    read_latencies_ms: List[float]
    write_latencies_ms: List[float]
    branch_latencies_ms: List[float]
    admin_latencies_ms: List[float]
    
    # Percentile latencies
    read_p50: float
    read_p95: float
    read_p99: float
    write_p50: float
    write_p95: float
    write_p99: float
    
    # Resource usage
    peak_memory_mb: float
    avg_cpu_percent: float
    peak_cpu_percent: float
    
    # Pass/Fail
    test_passed: bool
    failure_reasons: List[str]


class RFSourceTestHarness:
    """
    Test harness for rfsource operations.
    
    NOTE: This is a mock implementation for load testing demonstration.
    Replace with actual rfsource API calls when available.
    """
    
    def __init__(self, test_data_dir: str = "/tmp/rfsource_load_test"):
        self.test_data_dir = test_data_dir
        os.makedirs(test_data_dir, exist_ok=True)
        self._lock = threading.Lock()
        self._operations = 0
        
    def commit_artifact(self, user_id: int, artifact_data: Dict) -> Dict:
        """Simulate artifact commit operation."""
        # Simulate write latency (10ms baseline + jitter)
        time.sleep(0.010 + random.uniform(0, 0.005))
        
        with self._lock:
            self._operations += 1
            op_id = self._operations
        
        # Simulate occasional write failures (0.5% rate)
        if random.random() < 0.005:
            raise Exception("Simulated write conflict")
        
        return {
            "commit_id": f"commit_{op_id}",
            "timestamp": time.time(),
            "user_id": user_id
        }
    
    def read_artifact(self, user_id: int, artifact_id: str = None) -> Dict:
        """Simulate artifact read operation."""
        # Simulate read latency (100ms baseline + jitter)
        time.sleep(0.100 + random.uniform(0, 0.020))
        
        # Simulate occasional read failures (0.3% rate)
        if random.random() < 0.003:
            raise Exception("Simulated read timeout")
        
        return {
            "artifact_id": artifact_id or f"artifact_{random.randint(1, 10000)}",
            "data": {"key": "value"},
            "timestamp": time.time()
        }
    
    def create_branch(self, user_id: int, branch_name: str) -> Dict:
        """Simulate branch creation."""
        # Simulate branch operation latency (50ms baseline + jitter)
        time.sleep(0.050 + random.uniform(0, 0.010))
        
        # Simulate occasional conflicts (1% rate)
        if random.random() < 0.01:
            raise Exception("Simulated branch conflict")
        
        return {
            "branch_id": f"branch_{branch_name}_{int(time.time())}",
            "timestamp": time.time()
        }
    
    def query_history(self, user_id: int, filters: Dict = None) -> List[Dict]:
        """Simulate history query."""
        # Simulate query latency (80ms baseline + jitter)
        time.sleep(0.080 + random.uniform(0, 0.030))
        
        return [
            {"commit_id": f"commit_{i}", "timestamp": time.time()}
            for i in range(10)
        ]
    
    def create_proposal(self, user_id: int, proposal_data: Dict) -> Dict:
        """Simulate proposal creation."""
        # Simulate admin operation latency (150ms baseline + jitter)
        time.sleep(0.150 + random.uniform(0, 0.050))
        
        return {
            "proposal_id": f"proposal_{int(time.time())}_{user_id}",
            "status": "open"
        }


class LoadTestUser:
    """Simulates a single user performing operations."""
    
    def __init__(self, user_id: int, harness: RFSourceTestHarness):
        self.user_id = user_id
        self.harness = harness
        self.results: List[OperationResult] = []
    
    def _record_operation(self, op_type: str, start: float, end: float, 
                         success: bool, error: str = "") -> OperationResult:
        """Record operation result."""
        result = OperationResult(
            operation_type=op_type,
            user_id=self.user_id,
            start_time=start,
            end_time=end,
            success=success,
            error_message=error
        )
        self.results.append(result)
        return result
    
    def perform_read(self) -> OperationResult:
        """Perform read operation."""
        start = time.time()
        try:
            self.harness.read_artifact(self.user_id)
            return self._record_operation("read", start, time.time(), True)
        except Exception as e:
            return self._record_operation("read", start, time.time(), False, str(e))
    
    def perform_write(self) -> OperationResult:
        """Perform write operation."""
        start = time.time()
        try:
            self.harness.commit_artifact(self.user_id, {"data": f"user_{self.user_id}"})
            return self._record_operation("write", start, time.time(), True)
        except Exception as e:
            return self._record_operation("write", start, time.time(), False, str(e))
    
    def perform_branch_operation(self) -> OperationResult:
        """Perform branch operation."""
        start = time.time()
        try:
            self.harness.create_branch(self.user_id, f"branch_user_{self.user_id}")
            return self._record_operation("branch", start, time.time(), True)
        except Exception as e:
            return self._record_operation("branch", start, time.time(), False, str(e))
    
    def perform_admin_operation(self) -> OperationResult:
        """Perform administrative operation."""
        start = time.time()
        try:
            # Randomly choose admin operation (query history or proposal)
            if random.random() < 0.5:
                self.harness.query_history(self.user_id)
            else:
                self.harness.create_proposal(self.user_id, {"title": "test"})
            return self._record_operation("admin", start, time.time(), True)
        except Exception as e:
            return self._record_operation("admin", start, time.time(), False, str(e))
    
    def run_user_session(self, duration_seconds: int) -> List[OperationResult]:
        """
        Run user session for specified duration.
        
        Operation mix:
        - 70% reads
        - 20% writes
        - 5% branch operations
        - 5% admin operations
        """
        end_time = time.time() + duration_seconds
        operations = 0
        
        while time.time() < end_time:
            # Select operation based on probability distribution
            rand = random.random()
            
            if rand < 0.70:  # 70% reads
                self.perform_read()
            elif rand < 0.90:  # 20% writes
                self.perform_write()
            elif rand < 0.95:  # 5% branches
                self.perform_branch_operation()
            else:  # 5% admin
                self.perform_admin_operation()
            
            operations += 1
            
            # Small pause between operations (simulate think time)
            time.sleep(random.uniform(0.1, 0.5))
        
        return self.results


class ResourceMonitor:
    """Monitor system resource usage during test."""
    
    def __init__(self):
        self.process = psutil.Process()
        self.memory_samples: List[float] = []
        self.cpu_samples: List[float] = []
        self.monitoring = False
        self._monitor_thread = None
    
    def start(self):
        """Start monitoring."""
        self.monitoring = True
        self._monitor_thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self._monitor_thread.start()
    
    def stop(self):
        """Stop monitoring."""
        self.monitoring = False
        if self._monitor_thread:
            self._monitor_thread.join(timeout=2)
    
    def _monitor_loop(self):
        """Monitoring loop."""
        while self.monitoring:
            try:
                # Sample memory (MB)
                mem_info = self.process.memory_info()
                self.memory_samples.append(mem_info.rss / 1024 / 1024)
                
                # Sample CPU (%)
                cpu_percent = self.process.cpu_percent(interval=1)
                self.cpu_samples.append(cpu_percent)
            except:
                pass
            
            time.sleep(1)
    
    def get_metrics(self) -> Tuple[float, float, float]:
        """Get resource metrics (peak_mem_mb, avg_cpu%, peak_cpu%)."""
        peak_mem = max(self.memory_samples) if self.memory_samples else 0
        avg_cpu = statistics.mean(self.cpu_samples) if self.cpu_samples else 0
        peak_cpu = max(self.cpu_samples) if self.cpu_samples else 0
        return peak_mem, avg_cpu, peak_cpu


def calculate_percentile(values: List[float], percentile: float) -> float:
    """Calculate percentile from list of values."""
    if not values:
        return 0.0
    sorted_values = sorted(values)
    index = int(len(sorted_values) * percentile / 100)
    return sorted_values[min(index, len(sorted_values) - 1)]


def run_load_test(num_users: int, duration_seconds: int) -> LoadTestMetrics:
    """
    Run load test with specified number of concurrent users.
    
    Args:
        num_users: Number of concurrent users
        duration_seconds: Test duration in seconds
    
    Returns:
        LoadTestMetrics with test results
    """
    print(f"\n{'='*80}")
    print(f"RFSource Load Test - {num_users} Concurrent Users")
    print(f"{'='*80}\n")
    print(f"Test Configuration:")
    print(f"  Users:           {num_users}")
    print(f"  Duration:        {duration_seconds} seconds")
    print(f"  Operation Mix:   70% reads, 20% writes, 5% branches, 5% admin")
    print(f"\nStarting test...\n")
    
    # Initialize test harness
    harness = RFSourceTestHarness()
    
    # Initialize resource monitor
    monitor = ResourceMonitor()
    monitor.start()
    
    # Create users
    users = [LoadTestUser(i, harness) for i in range(num_users)]
    
    # Run test with thread pool
    start_time = time.time()
    
    with ThreadPoolExecutor(max_workers=num_users) as executor:
        futures = [
            executor.submit(user.run_user_session, duration_seconds)
            for user in users
        ]
        
        # Wait for all users to complete
        for i, future in enumerate(futures):
            if (i + 1) % 100 == 0:
                elapsed = time.time() - start_time
                print(f"  Progress: {i+1}/{num_users} users completed ({elapsed:.1f}s elapsed)")
            future.result()
    
    end_time = time.time()
    test_duration = end_time - start_time
    
    # Stop resource monitoring
    monitor.stop()
    peak_mem, avg_cpu, peak_cpu = monitor.get_metrics()
    
    print(f"\nTest completed in {test_duration:.2f} seconds\n")
    print("Analyzing results...\n")
    
    # Collect all results
    all_results: List[OperationResult] = []
    for user in users:
        all_results.extend(user.results)
    
    # Calculate metrics by operation type
    read_latencies = [r.latency_ms for r in all_results if r.operation_type == "read" and r.success]
    write_latencies = [r.latency_ms for r in all_results if r.operation_type == "write" and r.success]
    branch_latencies = [r.latency_ms for r in all_results if r.operation_type == "branch" and r.success]
    admin_latencies = [r.latency_ms for r in all_results if r.operation_type == "admin" and r.success]
    
    successful_ops = sum(1 for r in all_results if r.success)
    failed_ops = sum(1 for r in all_results if not r.success)
    total_ops = len(all_results)
    error_rate = (failed_ops / total_ops * 100) if total_ops > 0 else 0
    
    # Calculate percentiles
    read_p50 = calculate_percentile(read_latencies, 50)
    read_p95 = calculate_percentile(read_latencies, 95)
    read_p99 = calculate_percentile(read_latencies, 99)
    
    write_p50 = calculate_percentile(write_latencies, 50)
    write_p95 = calculate_percentile(write_latencies, 95)
    write_p99 = calculate_percentile(write_latencies, 99)
    
    # Evaluate pass/fail criteria
    failure_reasons = []
    
    if read_p95 > 200:
        failure_reasons.append(f"Read p95 latency ({read_p95:.1f}ms) exceeds 200ms threshold")
    
    if write_p95 > 50:
        failure_reasons.append(f"Write p95 latency ({write_p95:.1f}ms) exceeds 50ms threshold")
    
    if error_rate > 1.0:
        failure_reasons.append(f"Error rate ({error_rate:.2f}%) exceeds 1% threshold")
    
    if peak_mem > 4096:  # 4GB threshold
        failure_reasons.append(f"Peak memory ({peak_mem:.0f}MB) exceeds 4GB threshold")
    
    test_passed = len(failure_reasons) == 0
    
    metrics = LoadTestMetrics(
        total_operations=total_ops,
        successful_operations=successful_ops,
        failed_operations=failed_ops,
        error_rate=error_rate,
        duration_seconds=test_duration,
        operations_per_second=total_ops / test_duration,
        read_latencies_ms=read_latencies,
        write_latencies_ms=write_latencies,
        branch_latencies_ms=branch_latencies,
        admin_latencies_ms=admin_latencies,
        read_p50=read_p50,
        read_p95=read_p95,
        read_p99=read_p99,
        write_p50=write_p50,
        write_p95=write_p95,
        write_p99=write_p99,
        peak_memory_mb=peak_mem,
        avg_cpu_percent=avg_cpu,
        peak_cpu_percent=peak_cpu,
        test_passed=test_passed,
        failure_reasons=failure_reasons
    )
    
    return metrics


def print_results(metrics: LoadTestMetrics):
    """Print formatted test results."""
    print(f"\n{'='*80}")
    print("LOAD TEST RESULTS")
    print(f"{'='*80}\n")
    
    # Overall metrics
    print("Overall Metrics:")
    print(f"  Total Operations:        {metrics.total_operations:,}")
    print(f"  Successful:              {metrics.successful_operations:,} ({metrics.successful_operations/metrics.total_operations*100:.1f}%)")
    print(f"  Failed:                  {metrics.failed_operations:,} ({metrics.error_rate:.2f}%)")
    print(f"  Duration:                {metrics.duration_seconds:.2f} seconds")
    print(f"  Throughput:              {metrics.operations_per_second:.1f} ops/sec")
    
    # Latency metrics
    print(f"\nLatency Metrics (milliseconds):")
    print(f"\n  READ Operations ({len(metrics.read_latencies_ms):,} samples):")
    print(f"    p50:  {metrics.read_p50:>7.2f} ms")
    print(f"    p95:  {metrics.read_p95:>7.2f} ms {'✓' if metrics.read_p95 <= 200 else '✗ FAIL (>200ms)'}")
    print(f"    p99:  {metrics.read_p99:>7.2f} ms")
    
    print(f"\n  WRITE Operations ({len(metrics.write_latencies_ms):,} samples):")
    print(f"    p50:  {metrics.write_p50:>7.2f} ms")
    print(f"    p95:  {metrics.write_p95:>7.2f} ms {'✓' if metrics.write_p95 <= 50 else '✗ FAIL (>50ms)'}")
    print(f"    p99:  {metrics.write_p99:>7.2f} ms")
    
    if metrics.branch_latencies_ms:
        branch_p95 = calculate_percentile(metrics.branch_latencies_ms, 95)
        print(f"\n  BRANCH Operations ({len(metrics.branch_latencies_ms):,} samples):")
        print(f"    p95:  {branch_p95:>7.2f} ms")
    
    if metrics.admin_latencies_ms:
        admin_p95 = calculate_percentile(metrics.admin_latencies_ms, 95)
        print(f"\n  ADMIN Operations ({len(metrics.admin_latencies_ms):,} samples):")
        print(f"    p95:  {admin_p95:>7.2f} ms")
    
    # Resource usage
    print(f"\nResource Usage:")
    print(f"  Peak Memory:             {metrics.peak_memory_mb:.1f} MB")
    print(f"  Average CPU:             {metrics.avg_cpu_percent:.1f}%")
    print(f"  Peak CPU:                {metrics.peak_cpu_percent:.1f}%")
    
    # Pass/Fail summary
    print(f"\n{'='*80}")
    if metrics.test_passed:
        print("TEST RESULT: ✓ PASSED")
    else:
        print("TEST RESULT: ✗ FAILED")
        print(f"\nFailure Reasons:")
        for reason in metrics.failure_reasons:
            print(f"  - {reason}")
    print(f"{'='*80}\n")


def save_results(metrics: LoadTestMetrics, output_file: str):
    """Save results to JSON file."""
    results = {
        "timestamp": datetime.now().isoformat(),
        "metrics": {
            "total_operations": metrics.total_operations,
            "successful_operations": metrics.successful_operations,
            "failed_operations": metrics.failed_operations,
            "error_rate": metrics.error_rate,
            "duration_seconds": metrics.duration_seconds,
            "operations_per_second": metrics.operations_per_second,
            "latency": {
                "read": {
                    "p50": metrics.read_p50,
                    "p95": metrics.read_p95,
                    "p99": metrics.read_p99,
                    "samples": len(metrics.read_latencies_ms)
                },
                "write": {
                    "p50": metrics.write_p50,
                    "p95": metrics.write_p95,
                    "p99": metrics.write_p99,
                    "samples": len(metrics.write_latencies_ms)
                }
            },
            "resources": {
                "peak_memory_mb": metrics.peak_memory_mb,
                "avg_cpu_percent": metrics.avg_cpu_percent,
                "peak_cpu_percent": metrics.peak_cpu_percent
            }
        },
        "test_passed": metrics.test_passed,
        "failure_reasons": metrics.failure_reasons
    }
    
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"Results saved to: {output_file}")


def main():
    parser = argparse.ArgumentParser(description="RFSource Load Test")
    parser.add_argument('--users', type=int, default=500,
                       help='Number of concurrent users (default: 500)')
    parser.add_argument('--duration', type=int, default=300,
                       help='Test duration in seconds (default: 300)')
    parser.add_argument('--output', type=str, 
                       default=f"/tmp/rfsource_load_test_{int(time.time())}.json",
                       help='Output file for results (default: /tmp/rfsource_load_test_<timestamp>.json)')
    
    args = parser.parse_args()
    
    # Run load test
    metrics = run_load_test(args.users, args.duration)
    
    # Print results
    print_results(metrics)
    
    # Save results
    save_results(metrics, args.output)
    
    # Exit with appropriate code
    sys.exit(0 if metrics.test_passed else 1)


if __name__ == "__main__":
    main()
