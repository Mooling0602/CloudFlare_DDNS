// 简单的按时间间隔运行的函数
use tokio::time;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Local};

pub async fn run_with_schedule<F, Fut>(interval_seconds: u64, job_func: F) 
where 
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + 'static
{
    let duration = Duration::from_secs(interval_seconds);
    
    println!("定时任务已启动，执行间隔: {} 秒", interval_seconds);
    
    let mut execution_count = 0;
    
    loop {
        execution_count += 1;
        let start_time = SystemTime::now();
        let datetime: DateTime<Local> = start_time.into();
        
        println!("=== 第 {} 次执行开始 ===", execution_count);
        println!("执行时间: {}", datetime.format("%Y-%m-%d %H:%M:%S"));
        
        // 执行任务
        let task_result = job_func().await;
        
        // 计算任务执行时间
        let end_time = SystemTime::now();
        let elapsed = end_time.duration_since(start_time)
            .unwrap_or(Duration::from_secs(0));
        
        match task_result {
            Ok(()) => println!("定时任务执行成功 (耗时: {:.2}秒)", elapsed.as_secs_f64()),
            Err(e) => eprintln!("定时任务执行失败 (耗时: {:.2}秒): {}", elapsed.as_secs_f64(), e),
        }
        
        // 如果任务执行时间超过间隔时间，立即开始下一次执行
        // 否则等待剩余的时间
        if elapsed < duration {
            let wait_time = duration - elapsed;
            let next_execution = SystemTime::now() + wait_time;
            let next_datetime: DateTime<Local> = next_execution.into();
            println!("下一次执行时间: {}", next_datetime.format("%Y-%m-%d %H:%M:%S"));
            println!("等待 {:.2} 秒...", wait_time.as_secs_f64());
            time::sleep(wait_time).await;
        } else {
            println!("任务执行时间 ({:.2}秒) 超过间隔时间 ({}秒)，立即开始下一次执行", elapsed.as_secs_f64(), interval_seconds);
        }
    }
}