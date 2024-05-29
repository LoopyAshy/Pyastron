use std::{ffi::{ c_char, c_int, CString }, ptr::null};

use pyo3::{pyclass, pyfunction, pymethods, pymodule, types::{PyDict, PyDictMethods, PyModule}, wrap_pyfunction, Bound, PyObject, PyResult, Python};

#[link(name = "astrond", kind = "static", modifiers = "+whole-archive")]
extern "C" {
     fn run_astrond(cfgFile: *const c_char, prettyPrint: bool, logging: bool, logLevel: *const c_char, consoleLogging: bool) -> c_int;
     fn close_astrond(exit_code: c_int, throw_exception: bool);
}

#[pymodule]
pub fn pyastron(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    _py.run_bound("import multiprocessing", None, None)?;
    m.add_function(wrap_pyfunction!(loop_fn, m)?)?;
    m.add_function(wrap_pyfunction!(start_astron_direct, m)?)?;
    m.add_function(wrap_pyfunction!(close_astron_direct, m)?)?;
    m.add_function(wrap_pyfunction!(create, m)?)
}

#[pyfunction]
#[pyo3(signature = (cfg_file = "", pretty_print = false, logging = false, log_level = "", console_logging = true))]
/// Creates a new astron process. you will be required to run the `start` method on the returned process object to actually begin it.
pub fn create<'py>(py: Python<'py>, cfg_file: &str, pretty_print: bool, logging: bool, log_level: &str, console_logging: bool) -> PyResult<AstronProcess> {
    let locals = PyDict::new_bound(py);
    let queue = py.eval_bound(r#"multiprocessing.Queue()"#, None, None)?;
    locals.set_item("queue", queue.clone())?;
    locals.set_item("cfg_file", cfg_file)?;
    locals.set_item("pretty_print", pretty_print)?;
    locals.set_item("logging", logging)?;
    locals.set_item("log_level", log_level)?;
    locals.set_item("console_logging", console_logging)?;
    let process = py.eval_bound(r#"multiprocessing.Process(target=pyastron.loop_fn, args=(queue, cfg_file, pretty_print, logging, log_level, console_logging))"#, None, Some(&locals))?;
    Ok(AstronProcess {
        process: process.into(),
        queue: queue.into(),
        closed: false,
    })
}

#[pyfunction]
#[pyo3(signature = (queue, cfg_file = "", pretty_print = false, logging = false, log_level = "", console_logging = true))]
pub fn loop_fn<'py>(py: Python<'py>, queue: PyObject, cfg_file: &str, pretty_print: bool, logging: bool, log_level: &str, console_logging: bool) -> PyResult<()> {
    let cfg_file = String::from(cfg_file);
    let log_level = String::from(log_level);
    std::thread::spawn(move || {
        _ = start_astron_direct(&cfg_file, pretty_print, logging, &log_level, console_logging);
    });
    while queue.call_method0(py, "empty")?.extract(py).unwrap_or(true) {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    close_astron_direct(0, false)
}

#[pyclass]
pub struct AstronProcess {
    process: PyObject,
    queue: PyObject,
    closed: bool
}

#[pymethods]
impl AstronProcess {
    pub fn start(&self, py: Python<'_>) -> PyResult<()> {
        self.process.call_method0(py, "start")?;
        Ok(())
    }
    pub fn shutdown(&mut self, py: Python<'_>) -> PyResult<()> {
        if self.closed {
            return Err(pyo3::exceptions::PyException::new_err("Process is already closed"));
        }
        self.closed = true;
        self.queue.call_method1(py, "put", (1,))?;
        Ok(())
    }
    pub fn is_alive(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.process.call_method0(py, "is_alive")
    }
    /// This is not recommended as it can lead to hard crashes on the astron side.
    pub fn terminate(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.process.call_method0(py, "terminate")
    }
}


#[pyfunction]
#[pyo3(signature = (cfg_file = "", pretty_print = false, logging = false, log_level = "", console_logging = true))]
pub fn start_astron_direct(cfg_file: &str, pretty_print: bool, logging: bool, log_level: &str, console_logging: bool) -> PyResult<()> {
    let cfg_file = String::from(cfg_file);
    let log_level = String::from(log_level);
    unsafe {
        let cfg_file_cstr = if !cfg_file.is_empty() { Some(CString::new(cfg_file).expect("Failed to convert cfg_file to CString")) } else { None };
        let log_level_cstr = if !log_level.is_empty() { Some(CString::new(log_level).expect("Failed to convert log_level to CString")) } else { None };
        run_astrond(if let Some(cfg_file_cstr) = &cfg_file_cstr { cfg_file_cstr.as_ptr() } else { null() }, pretty_print, logging, if let Some(log_level_cstr) = &log_level_cstr { log_level_cstr.as_ptr() } else { null() }, console_logging);
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (exit_code, throw_exception = true))]
pub fn close_astron_direct(exit_code: c_int, throw_exception: bool) -> PyResult<()> {
    unsafe { close_astrond(exit_code, throw_exception); }
    Ok(())
}