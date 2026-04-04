use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
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
fn insert_sort<T: PartialOrd + Copy>(array: &mut Vec<T>) -> Result<(), Box<dyn std::error::Error>> {
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
fn get_variation_series<T: PartialEq + Copy>(array: &Vec<T>) -> Result<Vec<(T, usize)>, Box<dyn std::error::Error>> {
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

fn print_variation_graph(data: &[f64], intervals: usize) {
    if data.is_empty() { return; }

    let min = data[0];
    let max = data[data.len() - 1];
    let range = max - min;
    let step = if range == 0.0 { 1.0 } else { range / intervals as f64 };

    // 1. Считаем частоты
    let mut counts = vec![0; intervals];
    for &x in data {
        let mut idx = ((x - min) / step).floor() as usize;
        if idx >= intervals { idx = intervals - 1; }
        counts[idx] += 1;
    }

    let max_count = *counts.iter().max().unwrap_or(&0);

    // 2. Рисуем столбцы
    for level in (1..=max_count).rev() {
        print!("{:>2} │", level);
        for &count in &counts {
            if count >= level { print!("  █  "); } else { print!("     "); }
        }
        println!();
    }

    // 3. Рисуем ось
    println!("   └{}", "─────".repeat(intervals));

    // 4. Печатаем только Min и Max под крайними столбцами
    let min_str = format!("{:.1}", min);
    let max_str = format!("{:.1}", max);

    print!("     "); // Отступ оси Y
    
    // Печатаем Min под первым столбцом (центрировано в 5 символах)
    print!("{:^5}", min_str);

    // Печатаем пробелы до последнего столбца
    if intervals > 1 {
        print!("{}", "     ".repeat(intervals - 2));
        // Печатаем Max под последним столбцом
        print!("{:^5}", max_str);
    }
    
    println!("\n");
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
    
    // 1. Сортируем данные (обязательно для корректного поиска min/max)
    insert_sort(&mut stat)?;
    
    // 2. Выводим график, передавая исходный вектор чисел и количество интервалов
    // get_variation_series больше не нужен для этой функции
    print_variation_graph(&stat, 20);

    Ok(())
}
