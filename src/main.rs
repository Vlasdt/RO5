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


fn print_variation_graph<T: std::fmt::Display>(series: &[(T, usize)]) {
    let term_width = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80 // Значение по умолчанию, если терминал не определен
    };

    let col_width = 5;
    let axis_width = 5;
    let chunk_size = (term_width - axis_width) / col_width;

    if chunk_size == 0 { return; }

    for chunk in series.chunks(chunk_size) {
        let max_count = chunk.iter().map(|&(_, count)| count).max().unwrap_or(0);

        for level in (1..=max_count).rev() {
            print!("{:>2} │ ", level);
            for &(_, count) in chunk {
                if count >= level { print!("  █  "); } else { print!("     "); }
            }
            println!();
        }

        print!("   └{}", "─────".repeat(chunk.len()));
        print!("\n     ");
        for (val, _) in chunk { print!("{:^5}", val); }
        println!("\n");
    }
}




fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stat = Vec::new();  // Vec<f64> - одномерный вектор
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        for file_address in &args[1..] {
            let numbers = read_numbers_from_file(file_address)?;
            stat.extend(numbers);
        }
    } else {
        return Err("Need file path as argument".to_string().into());
    }
    
    // Сортируем данные
    insert_sort(&mut stat)?;
    
    // Строим вариационный ряд из уже отсортированных данных
    let series = get_variation_series(&stat)?;
    
    // Выводим график
    print_variation_graph(&series);

    Ok(())
}