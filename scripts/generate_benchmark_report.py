#!/usr/bin/env python3

"""
Benchmark report generator for Amp game engine
Analyzes benchmark results and generates performance reports
"""

import os
import sys
import json
import glob
from datetime import datetime
from typing import Dict, List, Any

class BenchmarkAnalyzer:
    def __init__(self, results_dir: str):
        self.results_dir = results_dir
        self.results = []
        
    def load_results(self):
        """Load all benchmark result files"""
        pattern = os.path.join(self.results_dir, "benchmark_*.json")
        files = glob.glob(pattern)
        
        for file in files:
            try:
                with open(file, 'r') as f:
                    data = json.load(f)
                    data['file'] = os.path.basename(file)
                    self.results.append(data)
            except Exception as e:
                print(f"Error loading {file}: {e}")
                
    def analyze_performance(self, result: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze performance metrics from a benchmark result"""
        metrics = result.get('metrics', {})
        
        frame_times = metrics.get('frame_times', [])
        if not frame_times:
            return {}
            
        # Calculate statistics
        avg_frame_time = sum(frame_times) / len(frame_times)
        min_frame_time = min(frame_times)
        max_frame_time = max(frame_times)
        
        # Calculate percentiles
        sorted_times = sorted(frame_times)
        p95_index = int(len(sorted_times) * 0.95)
        p99_index = int(len(sorted_times) * 0.99)
        
        p95_frame_time = sorted_times[p95_index]
        p99_frame_time = sorted_times[p99_index]
        
        # Calculate FPS
        avg_fps = 1000.0 / avg_frame_time if avg_frame_time > 0 else 0
        min_fps = 1000.0 / max_frame_time if max_frame_time > 0 else 0
        
        return {
            'avg_frame_time_ms': avg_frame_time,
            'min_frame_time_ms': min_frame_time,
            'max_frame_time_ms': max_frame_time,
            'p95_frame_time_ms': p95_frame_time,
            'p99_frame_time_ms': p99_frame_time,
            'avg_fps': avg_fps,
            'min_fps': min_fps,
            'frame_count': len(frame_times),
            'gpu_culling_time_ms': metrics.get('gpu_culling_time_ms', 0),
            'batch_processing_time_ms': metrics.get('batch_processing_time_ms', 0),
            'lod_update_time_ms': metrics.get('lod_update_time_ms', 0),
            'streaming_time_ms': metrics.get('streaming_time_ms', 0),
            'visible_instances': metrics.get('visible_instances', 0),
            'culled_instances': metrics.get('culled_instances', 0),
            'batch_count': metrics.get('batch_count', 0),
            'memory_usage_mb': metrics.get('memory_usage_mb', 0),
        }
        
    def generate_report(self):
        """Generate a comprehensive performance report"""
        if not self.results:
            print("No benchmark results found")
            return
            
        print("=== Amp Game Engine Benchmark Report ===")
        print(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"Results Directory: {self.results_dir}")
        print(f"Total Benchmarks: {len(self.results)}")
        print()
        
        # Performance targets
        targets = {
            'target_fps': 60,
            'warning_fps': 30,
            'critical_fps': 15,
            'target_frame_time_ms': 16.67,
            'warning_frame_time_ms': 33.33,
            'critical_frame_time_ms': 66.67,
            'gpu_culling_budget_ms': 0.25,
            'batch_processing_budget_ms': 2.5,
            'lod_update_budget_ms': 1.0,
            'streaming_budget_ms': 1.5,
        }
        
        # Analyze each benchmark
        for result in self.results:
            scene_name = result.get('scene', 'unknown')
            analysis = self.analyze_performance(result)
            
            if not analysis:
                continue
                
            print(f"=== Scene: {scene_name} ===")
            print(f"File: {result['file']}")
            print(f"Frames: {analysis['frame_count']}")
            print()
            
            # Frame time analysis
            print("Frame Time Analysis:")
            print(f"  Average: {analysis['avg_frame_time_ms']:.2f}ms")
            print(f"  P95: {analysis['p95_frame_time_ms']:.2f}ms")
            print(f"  P99: {analysis['p99_frame_time_ms']:.2f}ms")
            print(f"  Min: {analysis['min_frame_time_ms']:.2f}ms")
            print(f"  Max: {analysis['max_frame_time_ms']:.2f}ms")
            print()
            
            # FPS analysis
            print("FPS Analysis:")
            print(f"  Average: {analysis['avg_fps']:.1f} FPS")
            print(f"  Minimum: {analysis['min_fps']:.1f} FPS")
            print()
            
            # System performance
            print("System Performance:")
            print(f"  GPU Culling: {analysis['gpu_culling_time_ms']:.2f}ms")
            print(f"  Batch Processing: {analysis['batch_processing_time_ms']:.2f}ms")
            print(f"  LOD Updates: {analysis['lod_update_time_ms']:.2f}ms")
            print(f"  Streaming: {analysis['streaming_time_ms']:.2f}ms")
            print()
            
            # Resource usage
            print("Resource Usage:")
            print(f"  Visible Instances: {analysis['visible_instances']:,}")
            print(f"  Culled Instances: {analysis['culled_instances']:,}")
            print(f"  Batch Count: {analysis['batch_count']:,}")
            print(f"  Memory Usage: {analysis['memory_usage_mb']:.1f} MB")
            print()
            
            # Performance assessment
            print("Performance Assessment:")
            self.assess_performance(analysis, targets)
            print()
            print("-" * 60)
            print()
            
    def assess_performance(self, analysis: Dict[str, Any], targets: Dict[str, float]):
        """Assess performance against targets"""
        assessments = []
        
        # Frame time assessment
        avg_frame_time = analysis['avg_frame_time_ms']
        if avg_frame_time <= targets['target_frame_time_ms']:
            assessments.append("✓ Frame time: EXCELLENT")
        elif avg_frame_time <= targets['warning_frame_time_ms']:
            assessments.append("⚠ Frame time: ACCEPTABLE")
        else:
            assessments.append("✗ Frame time: POOR")
            
        # FPS assessment
        avg_fps = analysis['avg_fps']
        if avg_fps >= targets['target_fps']:
            assessments.append("✓ FPS: EXCELLENT")
        elif avg_fps >= targets['warning_fps']:
            assessments.append("⚠ FPS: ACCEPTABLE")
        else:
            assessments.append("✗ FPS: POOR")
            
        # GPU culling assessment
        gpu_time = analysis['gpu_culling_time_ms']
        if gpu_time <= targets['gpu_culling_budget_ms']:
            assessments.append("✓ GPU Culling: WITHIN BUDGET")
        else:
            assessments.append("✗ GPU Culling: OVER BUDGET")
            
        # Batch processing assessment
        batch_time = analysis['batch_processing_time_ms']
        if batch_time <= targets['batch_processing_budget_ms']:
            assessments.append("✓ Batch Processing: WITHIN BUDGET")
        else:
            assessments.append("✗ Batch Processing: OVER BUDGET")
            
        # LOD update assessment
        lod_time = analysis['lod_update_time_ms']
        if lod_time <= targets['lod_update_budget_ms']:
            assessments.append("✓ LOD Updates: WITHIN BUDGET")
        else:
            assessments.append("✗ LOD Updates: OVER BUDGET")
            
        # Streaming assessment
        streaming_time = analysis['streaming_time_ms']
        if streaming_time <= targets['streaming_budget_ms']:
            assessments.append("✓ Streaming: WITHIN BUDGET")
        else:
            assessments.append("✗ Streaming: OVER BUDGET")
            
        for assessment in assessments:
            print(f"  {assessment}")
            
    def save_summary(self):
        """Save benchmark summary to JSON file"""
        summary = {
            'timestamp': datetime.now().isoformat(),
            'results_dir': self.results_dir,
            'benchmark_count': len(self.results),
            'benchmarks': []
        }
        
        for result in self.results:
            analysis = self.analyze_performance(result)
            if analysis:
                summary['benchmarks'].append({
                    'scene': result.get('scene', 'unknown'),
                    'file': result['file'],
                    'analysis': analysis
                })
                
        summary_file = os.path.join(self.results_dir, 'benchmark_summary.json')
        with open(summary_file, 'w') as f:
            json.dump(summary, f, indent=2)
            
        print(f"Summary saved to: {summary_file}")

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 generate_benchmark_report.py <results_directory>")
        sys.exit(1)
        
    results_dir = sys.argv[1]
    
    if not os.path.exists(results_dir):
        print(f"Results directory does not exist: {results_dir}")
        sys.exit(1)
        
    analyzer = BenchmarkAnalyzer(results_dir)
    analyzer.load_results()
    analyzer.generate_report()
    analyzer.save_summary()

if __name__ == "__main__":
    main()
