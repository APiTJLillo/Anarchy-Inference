"""
Resource Monitor for Anarchy Inference Stress Testing

This module provides classes for monitoring resource usage during stress tests,
including memory, CPU, file handles, and network connections.
"""

import os
import sys
import time
import threading
import psutil
import gc
from typing import Dict, List, Any, Optional
from dataclasses import dataclass

# Add the parent directory to the path so we can import the anarchy module
sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Make sure it's in the parent directory.")

class ResourceMonitor:
    """Monitors resource usage during stress tests."""
    
    def __init__(self, sampling_interval: float = 0.5):
        """Initialize the resource monitor.
        
        Args:
            sampling_interval: Time between resource usage samples in seconds
        """
        self.sampling_interval = sampling_interval
        self.running = False
        self.thread = None
        self.process = psutil.Process(os.getpid())
        
        # Resource usage history
        self.memory_usage = []
        self.cpu_usage = []
        self.file_handles = []
        self.thread_count = []
        self.gc_stats = []
        
        # Peak values
        self.peak_memory = 0
        self.peak_cpu = 0
        self.peak_file_handles = 0
        self.peak_thread_count = 0
    
    def start(self):
        """Start monitoring resource usage."""
        if self.running:
            return
        
        self.running = True
        self.thread = threading.Thread(target=self._monitor_resources)
        self.thread.daemon = True
        self.thread.start()
    
    def stop(self) -> Dict[str, Any]:
        """Stop monitoring resource usage and return the results."""
        if not self.running:
            return self._get_resource_summary()
        
        self.running = False
        if self.thread:
            self.thread.join(timeout=2.0)
        
        return self._get_resource_summary()
    
    def _monitor_resources(self):
        """Monitor resource usage in a background thread."""
        while self.running:
            try:
                # Memory usage
                memory_info = self.process.memory_info()
                memory_mb = memory_info.rss / (1024 * 1024)  # Convert to MB
                self.memory_usage.append(memory_mb)
                self.peak_memory = max(self.peak_memory, memory_mb)
                
                # CPU usage
                cpu_percent = self.process.cpu_percent(interval=None)
                self.cpu_usage.append(cpu_percent)
                self.peak_cpu = max(self.peak_cpu, cpu_percent)
                
                # File handles
                try:
                    open_files = len(self.process.open_files())
                    self.file_handles.append(open_files)
                    self.peak_file_handles = max(self.peak_file_handles, open_files)
                except (psutil.AccessDenied, psutil.NoSuchProcess):
                    pass
                
                # Thread count
                thread_count = len(self.process.threads())
                self.thread_count.append(thread_count)
                self.peak_thread_count = max(self.peak_thread_count, thread_count)
                
                # GC stats
                gc_counts = gc.get_count()
                self.gc_stats.append(gc_counts)
                
                # Sleep until next sample
                time.sleep(self.sampling_interval)
            
            except Exception as e:
                print(f"Error in resource monitoring: {e}")
                break
    
    def _get_resource_summary(self) -> Dict[str, Any]:
        """Get a summary of resource usage."""
        # Calculate averages
        avg_memory = sum(self.memory_usage) / len(self.memory_usage) if self.memory_usage else 0
        avg_cpu = sum(self.cpu_usage) / len(self.cpu_usage) if self.cpu_usage else 0
        avg_file_handles = sum(self.file_handles) / len(self.file_handles) if self.file_handles else 0
        avg_thread_count = sum(self.thread_count) / len(self.thread_count) if self.thread_count else 0
        
        # Calculate memory growth
        memory_growth = (self.memory_usage[-1] - self.memory_usage[0]) if len(self.memory_usage) > 1 else 0
        
        # Count GC collections
        gc_collections = [0, 0, 0]
        for i in range(1, len(self.gc_stats)):
            for gen in range(3):
                if self.gc_stats[i][gen] < self.gc_stats[i-1][gen]:
                    gc_collections[gen] += 1
        
        return {
            "memory": {
                "peak_mb": self.peak_memory,
                "average_mb": avg_memory,
                "final_mb": self.memory_usage[-1] if self.memory_usage else 0,
                "growth_mb": memory_growth,
                "samples": len(self.memory_usage)
            },
            "cpu": {
                "peak_percent": self.peak_cpu,
                "average_percent": avg_cpu,
                "samples": len(self.cpu_usage)
            },
            "file_handles": {
                "peak": self.peak_file_handles,
                "average": avg_file_handles,
                "samples": len(self.file_handles)
            },
            "threads": {
                "peak": self.peak_thread_count,
                "average": avg_thread_count,
                "samples": len(self.thread_count)
            },
            "gc": {
                "collections_gen0": gc_collections[0],
                "collections_gen1": gc_collections[1],
                "collections_gen2": gc_collections[2]
            }
        }

class MemoryTracker:
    """Tracks memory allocations and deallocations to detect leaks."""
    
    def __init__(self, interpreter):
        """Initialize the memory tracker.
        
        Args:
            interpreter: The Anarchy Inference interpreter instance
        """
        self.interpreter = interpreter
        self.allocation_counts = {}
        self.deallocation_counts = {}
        self.active_objects = {}
        self.tracking_enabled = False
    
    def start_tracking(self):
        """Start tracking memory allocations and deallocations."""
        self.tracking_enabled = True
        # Hook into the interpreter's memory management system
        # This is a placeholder - the actual implementation would depend on
        # the interpreter's memory management API
    
    def stop_tracking(self) -> Dict[str, Any]:
        """Stop tracking memory and return the results."""
        self.tracking_enabled = False
        
        # Calculate potential leaks
        leaks = {}
        for obj_type, count in self.allocation_counts.items():
            deallocations = self.deallocation_counts.get(obj_type, 0)
            if count > deallocations:
                leaks[obj_type] = count - deallocations
        
        return {
            "allocations": dict(self.allocation_counts),
            "deallocations": dict(self.deallocation_counts),
            "active_objects": len(self.active_objects),
            "potential_leaks": leaks
        }
    
    def record_allocation(self, obj_id, obj_type):
        """Record an object allocation."""
        if not self.tracking_enabled:
            return
        
        self.allocation_counts[obj_type] = self.allocation_counts.get(obj_type, 0) + 1
        self.active_objects[obj_id] = obj_type
    
    def record_deallocation(self, obj_id):
        """Record an object deallocation."""
        if not self.tracking_enabled or obj_id not in self.active_objects:
            return
        
        obj_type = self.active_objects[obj_id]
        self.deallocation_counts[obj_type] = self.deallocation_counts.get(obj_type, 0) + 1
        del self.active_objects[obj_id]

class CPUMonitor:
    """Monitors CPU usage and identifies hotspots."""
    
    def __init__(self, sampling_interval: float = 0.1):
        """Initialize the CPU monitor.
        
        Args:
            sampling_interval: Time between CPU usage samples in seconds
        """
        self.sampling_interval = sampling_interval
        self.running = False
        self.thread = None
        self.process = psutil.Process(os.getpid())
        
        # CPU usage history
        self.cpu_usage = []
        self.cpu_times = []
        
        # Function execution times (would be populated by interpreter hooks)
        self.function_times = {}
    
    def start(self):
        """Start monitoring CPU usage."""
        if self.running:
            return
        
        self.running = True
        self.thread = threading.Thread(target=self._monitor_cpu)
        self.thread.daemon = True
        self.thread.start()
    
    def stop(self) -> Dict[str, Any]:
        """Stop monitoring CPU usage and return the results."""
        if not self.running:
            return self._get_cpu_summary()
        
        self.running = False
        if self.thread:
            self.thread.join(timeout=2.0)
        
        return self._get_cpu_summary()
    
    def _monitor_cpu(self):
        """Monitor CPU usage in a background thread."""
        while self.running:
            try:
                # CPU usage
                cpu_percent = self.process.cpu_percent(interval=None)
                self.cpu_usage.append(cpu_percent)
                
                # CPU times
                cpu_times = self.process.cpu_times()
                self.cpu_times.append({
                    "user": cpu_times.user,
                    "system": cpu_times.system,
                    "timestamp": time.time()
                })
                
                # Sleep until next sample
                time.sleep(self.sampling_interval)
            
            except Exception as e:
                print(f"Error in CPU monitoring: {e}")
                break
    
    def _get_cpu_summary(self) -> Dict[str, Any]:
        """Get a summary of CPU usage."""
        # Calculate statistics
        avg_cpu = sum(self.cpu_usage) / len(self.cpu_usage) if self.cpu_usage else 0
        peak_cpu = max(self.cpu_usage) if self.cpu_usage else 0
        
        # Calculate CPU time deltas
        user_time = 0
        system_time = 0
        if len(self.cpu_times) > 1:
            first = self.cpu_times[0]
            last = self.cpu_times[-1]
            user_time = last["user"] - first["user"]
            system_time = last["system"] - first["system"]
            total_time = last["timestamp"] - first["timestamp"]
        
        # Find hotspot functions
        hotspots = sorted(
            self.function_times.items(),
            key=lambda x: x[1],
            reverse=True
        )[:10]  # Top 10 hotspots
        
        return {
            "average_percent": avg_cpu,
            "peak_percent": peak_cpu,
            "user_time": user_time,
            "system_time": system_time,
            "samples": len(self.cpu_usage),
            "hotspots": dict(hotspots)
        }
    
    def record_function_time(self, function_name: str, execution_time: float):
        """Record the execution time of a function."""
        if function_name in self.function_times:
            self.function_times[function_name] += execution_time
        else:
            self.function_times[function_name] = execution_time

class FileHandleTracker:
    """Tracks file handle usage to detect leaks."""
    
    def __init__(self):
        """Initialize the file handle tracker."""
        self.process = psutil.Process(os.getpid())
        self.open_files = {}
        self.tracking_enabled = False
    
    def start_tracking(self):
        """Start tracking file handle usage."""
        self.tracking_enabled = True
        self._update_open_files()
    
    def stop_tracking(self) -> Dict[str, Any]:
        """Stop tracking file handles and return the results."""
        self.tracking_enabled = False
        self._update_open_files()
        
        return {
            "open_files": len(self.open_files),
            "files_by_type": self._count_files_by_type()
        }
    
    def _update_open_files(self):
        """Update the list of open files."""
        if not self.tracking_enabled:
            return
        
        try:
            current_files = self.process.open_files()
            self.open_files = {f.path: f for f in current_files}
        except (psutil.AccessDenied, psutil.NoSuchProcess):
            pass
    
    def _count_files_by_type(self) -> Dict[str, int]:
        """Count open files by file extension."""
        counts = {}
        for path in self.open_files:
            ext = os.path.splitext(path)[1].lower()
            if not ext:
                ext = "(no extension)"
            counts[ext] = counts.get(ext, 0) + 1
        return counts
