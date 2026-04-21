use core::error;
use std::{clone, env, iter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::Sum;
use std::path::Path;

fn read_numbers_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut numbers = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        for word in line.split_whitespace() {
            if let Ok(num) = word.parse::<f64>() {
                numbers.push(num);
            }
        }
    }
    
    Ok(numbers)
}

/// Эта функция сортирует вектор методом вставки
fn insert_sort<T: PartialOrd + Copy>(array: &mut [T]) -> Result<(), Box<dyn std::error::Error>> {
    if array.is_empty() {
        return Err("Cannot sort empty vector".to_string().into());
    }
    
    for i in 1..array.len() {
        let key = array[i];
        let mut j = i;
        
        while j > 0 && array[j - 1] > key {
            array[j] = array[j - 1];
            j -= 1;
        }
        
        array[j] = key;
    }
    Ok(())
}

/// Строит вариационный ряд из отсортированного массива данных.
fn get_variation_series<T: PartialEq + Copy>(array: &[T]) -> Result<Vec<(T, usize)>, Box<dyn std::error::Error>> {
    if array.is_empty() {
        return Err("array is empty".to_string().into());
    }

    let mut result = Vec::new();
    let mut current = array[0];
    let mut count = 1;
    
    for i in 1..array.len() {
        if array[i] == current {
            count += 1;
        } else {
            result.push((current, count));
            current = array[i];
            count = 1;
        }
    }
    
    result.push((current, count));
    Ok(result)
}

use terminal_size::{Width, terminal_size};

fn print_variation_graph(data_in: &[f64], intervals: usize) -> Result<(), Box<dyn std::error::Error>> {
    if data_in.is_empty() { 
        return Err("No data to plot".into());
    }
    
    // Создаем копию и сортируем только для графика
    let mut data = data_in.to_vec();
    insert_sort(&mut data)?;
    
    let min = data[0];
    let max = data[data.len() - 1];
    let range = max - min;
    let step = if range == 0.0 { 1.0 } else { range / intervals as f64 };

    let mut counts = vec![0; intervals];
    for &x in &data {
        let mut idx = ((x - min) / step).floor() as usize;
        if idx >= intervals { idx = intervals - 1; }
        counts[idx] += 1;
    }

    let max_count = *counts.iter().max().unwrap_or(&0);

    for level in (1..=max_count).rev() {
        print!("{:>2} │", level);
        for &count in &counts {
            if count >= level { print!("  █  "); } else { print!("     "); }
        }
        println!();
    }

    println!("   └{}", "─────".repeat(intervals));

    let min_str = format!("{:.1}", min);
    let max_str = format!("{:.1}", max);

    print!("     "); // Отступ оси Y
    
    print!("{:^5}", min_str);

    if intervals > 1 {
        print!("{}", "     ".repeat(intervals - 2));
        print!("{:^5}", max_str);
    }
    
    println!("\n");
    
    Ok(())
}


fn sample_mean<T: Sum + Copy + Into<f64>>(array: &Vec<T>) -> Result<f64, Box<dyn std::error::Error>> {
    let sum: f64 = array.iter().copied().map(|x| x.into()).sum();
    let len = array.len();
    if len == 0 {
        return Err("Cannot compute mean of empty array".into());
    }
    Ok(sum / len as f64)
}

fn mode<T: Clone + PartialOrd + Copy>(input: &[T]) -> Result<(Vec<T>, usize), Box<dyn std::error::Error>> {
    let mut sorted = input.to_vec();  // Создаем копию для сортировки
    insert_sort(&mut sorted)?;
    let vs = get_variation_series(&sorted)?;
    
    if vs.is_empty() {
        return Err("Cannot find mode of empty array".into());
    }
    
    let max_count = vs.iter().map(|(_, count)| *count).max().unwrap_or(0);
    
    let modes: Vec<T> = vs.iter()
        .filter(|(_, count)| *count == max_count)
        .map(|(value, _)| *value)
        .collect();
    
    Ok((modes, max_count))
}

fn median<T: Copy + PartialOrd + Into<f64>>(input: &[T]) -> Result<f64, Box<dyn std::error::Error>> {
    if input.is_empty() {
        return Err("Cannot compute median of empty array".into());
    }
    
    let mut sorted = input.to_vec();
    insert_sort(&mut sorted)?;
    
    let len = sorted.len();
    let mid = len / 2;
    
    if len % 2 == 0 {
        let left = sorted[mid - 1].into();
        let right = sorted[mid].into();
        Ok((left + right) / 2.0)
    } else {
        Ok(sorted[mid].into())
    }
}


use std::ops::Sub;

fn range<T: Copy + Clone + PartialOrd + Sub<Output = T>>(input: &Vec<T>) -> Result<T, Box<dyn std::error::Error>> {
    if input.is_empty() {
        return Err("Cannot compute range of empty array".into());
    }
    
    let mut sorted = input.clone();
    insert_sort(&mut sorted)?;
    
    let min = &sorted[0];
    let max = &sorted[sorted.len() - 1];
    
    Ok(max.clone() - min.clone())
}



/// межквартильный размах (IQR)
fn iqr<T: Copy + PartialOrd + Into<f64>>(input: &[T]) -> Result<f64, Box<dyn std::error::Error>> {

    
    let mut sorted = input.to_vec();
    insert_sort(&mut sorted)?;
    
    let q1 = quartile(&sorted, 0.25)?;
    let q3 = quartile(&sorted, 0.75)?;
    
    Ok(q3 - q1)
}

/// Вычисляет квартиль для отсортированных данных
fn quartile<T: Copy + PartialOrd + Into<f64>>(
    sorted_data: &[T],
    percentile: f64,
) -> Result<f64, Box<dyn std::error::Error>> {
    
    let n = sorted_data.len();
    let position = (n as f64 - 1.0) * percentile;
    let index = position.floor() as usize;
    let frac = position - index as f64;
    
    if index >= n - 1 {
        return Ok(sorted_data[n - 1].into());
    }
    
    let lower: f64 = sorted_data[index].into();
    let upper: f64 = sorted_data[index + 1].into();
    
    Ok(lower + frac * (upper - lower))
}


fn variance<T: Copy + Into<f64> + PartialEq>(data: &Vec<T>) -> Result<f64, Box<dyn std::error::Error>> {
    
    let variation_series = get_variation_series(&data)?; 
    let sum: f64 = data.iter().map(|&x| x.into()).sum();
    let mean = sum / data.len() as f64; // среднее значение 
    
    let sum_squared_diff: f64 = variation_series
        .iter()
        .map(|(value, count)| {
            let diff: f64 = (*value).into() - mean;
            diff * diff * (*count as f64)
        })
        .sum();
    
    Ok(sum_squared_diff / (data.len() - 1) as f64)
}

fn standard_error_mean<T: PartialEq + Copy + Into<f64>>(data: &Vec<T>) -> Result<f64, Box<dyn std::error::Error>> {

    Ok(variance(&data)?.sqrt() / (data.len() as f64).sqrt())
}

fn confidence_interval_95<T: Copy + Into<f64>  + Sum + PartialEq>(
    data: &Vec<T>,  // Изменено: &Vec<T> на &[T] для консистентности
) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let mean = sample_mean(data)?;  // data уже ссылка
    let sem = standard_error_mean(data)?;
    
    // Для 95% доверительного интервала используем z-score = 1.96
    let z = 1.96;
    let margin = z * sem;
    
    Ok((mean - margin, mean + margin))
}





fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stat = Vec::new();
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        for file_address in &args[1..] {
            let numbers = read_numbers_from_file(file_address)?;
            stat.extend(numbers);
        }
    } else {
        return Err("Need file path as argument".to_string().into());
    }
    
    println!("{:?}", &stat.iter().take(20).collect::<Vec<_>>());
    
    print_variation_graph(&stat, 20)?;
    
    println!("Sample mean: {}", sample_mean(&stat)?);
    
    let (modes, count) = mode(&stat)?;
    println!("Modes: {:?}, Frequency: {}", modes, count);
    
    println!("mediana: {}", median(&stat)?);

    println!("range: {}",range(&stat)?);

    println!("IQR: {}",iqr(&stat)?);
    
    println!("Дисперсия: {:.3}", variance(&stat)?);
    
    println!("среднее квадратическое отклонение: {}",variance(&stat)?.sqrt());

    println!("Коэффициент вариации: {:.2}%", (variance(&stat)?.sqrt() / sample_mean(&stat)?) * 100.0);

    println!("ошибку
репрезентативности для средней величины генеральной совокупности: {}", standard_error_mean(&stat)?);

    let (left,right) = confidence_interval_95(&stat)?;

    println!("доверительный интервал для средней величины с уровнем значимости α=0.05 : [{},{}]",left,right);

    println!("{:?}", &stat.iter().take(20).collect::<Vec<_>>());


    Ok(())
}