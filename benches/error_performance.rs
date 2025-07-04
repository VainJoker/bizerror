use std::{
    error::Error as StdError,
    hint::black_box,
};

use bizerror::*;
use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use thiserror::Error as ThisError;

#[derive(BizError, ThisError)]
pub enum BenchError {
    #[bizcode(1001)]
    #[error("Simple error")]
    SimpleError,

    #[bizcode(1002)]
    #[error("Error with data: {value}")]
    DataError { value: i32 },

    #[bizcode(1003)]
    #[error("Error with source: {0}")]
    SourceError(#[from] std::io::Error),
}

fn benchmark_error_creation(c: &mut Criterion) {
    c.bench_function("create_simple_error", |b| {
        b.iter(|| {
            let error = BenchError::SimpleError;
            black_box(error)
        });
    });

    c.bench_function("create_data_error", |b| {
        b.iter(|| {
            let error = BenchError::DataError { value: 42 };
            black_box(error)
        });
    });

    c.bench_function("create_source_error", |b| {
        b.iter(|| {
            let io_error =
                std::io::Error::new(std::io::ErrorKind::NotFound, "test");
            let error = BenchError::SourceError(io_error);
            black_box(error)
        });
    });
}

fn benchmark_error_methods(c: &mut Criterion) {
    c.bench_function("get_error_code", |b| {
        b.iter(|| {
            let error = BenchError::SimpleError;
            let code = error.code();
            black_box(code)
        });
    });

    c.bench_function("get_error_name", |b| {
        b.iter(|| {
            let error = BenchError::SimpleError;
            let name = error.name().to_string();
            black_box(name)
        });
    });

    c.bench_function("get_error_message", |b| {
        b.iter(|| {
            let error = BenchError::SimpleError;
            let msg = error.to_string();
            black_box(msg)
        });
    });

    c.bench_function("format_error_display", |b| {
        b.iter(|| {
            let error = BenchError::SimpleError;
            let display = format!("{error}");
            black_box(display)
        });
    });

    c.bench_function("format_error_debug", |b| {
        b.iter(|| {
            let error = BenchError::SimpleError;
            let debug = format!("{error:?}");
            black_box(debug)
        });
    });
}

fn benchmark_contextual_error(c: &mut Criterion) {
    c.bench_function("create_contextual_error", |b| {
        b.iter(|| {
            let base_error = BenchError::SimpleError;
            let error = base_error.with_context("Test context");
            black_box(error)
        });
    });

    c.bench_function("contextual_error_methods", |b| {
        b.iter(|| {
            let base_error = BenchError::SimpleError;
            let contextual = base_error.with_context("Test context");
            let code = contextual.code();
            let name = contextual.name().to_string();
            let context_str = contextual.context().to_string();
            let location_str = format!("{}", contextual.location());
            black_box((code, name, context_str, location_str))
        });
    });
}

fn benchmark_error_chain(c: &mut Criterion) {
    c.bench_function("create_error_chain", |b| {
        b.iter(|| {
            let io_error =
                std::io::Error::new(std::io::ErrorKind::NotFound, "test");
            let bench_error = BenchError::from(io_error);
            let contextual = bench_error.with_context("Operation failed");
            black_box(contextual)
        });
    });

    c.bench_function("traverse_error_chain", |b| {
        b.iter(|| {
            let io_error =
                std::io::Error::new(std::io::ErrorKind::NotFound, "test");
            let bench_error = BenchError::from(io_error);
            let contextual = bench_error.with_context("Operation failed");

            let mut source = StdError::source(&contextual);
            let mut count = 0;
            while let Some(err) = source {
                count += 1;
                source = StdError::source(err);
            }
            black_box(count)
        });
    });
}

fn benchmark_result_ext(c: &mut Criterion) {
    fn failing_operation() -> Result<String, std::io::Error> {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "test"))
    }

    c.bench_function("result_with_context", |b| {
        b.iter(|| {
            let result: Result<String, ContextualError<BenchError>> =
                failing_operation().with_context("Operation context");
            black_box(result)
        });
    });
}

fn benchmark_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_simple_error", |b| {
        b.iter(|| {
            let errors: Vec<BenchError> =
                (0..1000).map(|_| BenchError::SimpleError).collect();
            black_box(errors)
        });
    });

    c.bench_function("memory_contextual_error", |b| {
        b.iter(|| {
            let errors: Vec<ContextualError<BenchError>> = (0..1000)
                .map(|i| {
                    BenchError::SimpleError.with_context(format!("Context {i}"))
                })
                .collect();
            black_box(errors)
        });
    });
}

criterion_group!(
    benches,
    benchmark_error_creation,
    benchmark_error_methods,
    benchmark_contextual_error,
    benchmark_error_chain,
    benchmark_result_ext,
    benchmark_memory_usage
);

criterion_main!(benches);
