use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use elements::Module;
use interpreter::{Error, CustomUserError, InterpreterError};
use interpreter::env::{self, env_module};
use interpreter::module::{ModuleInstance, ModuleInstanceInterface};

/// Program instance. Program is a set of instantiated modules.
pub struct ProgramInstance<E: CustomUserError> {
	/// Shared data reference.
	essence: Arc<ProgramInstanceEssence<E>>,
}

/// Program instance essence.
pub struct ProgramInstanceEssence<E: CustomUserError> {
	/// Loaded modules.
	modules: RwLock<HashMap<String, Arc<ModuleInstanceInterface<E>>>>,
}

impl<E> ProgramInstance<E> where E: CustomUserError {
	/// Create new program instance.
	pub fn new() -> Result<Self, Error> {
		ProgramInstance::with_env_params(env::EnvParams::default())
	}

	/// Create new program instance with custom env module params (mostly memory)
	pub fn with_env_params(params: env::EnvParams) -> Result<Self, Error> {
		Ok(ProgramInstance {
			essence: Arc::new(ProgramInstanceEssence::with_env_params(params)?),
		})
	}

	/// Instantiate module with validation.
	pub fn add_module<'a>(&self, name: &str, module: Module, externals: Option<&'a HashMap<String, Arc<ModuleInstanceInterface<E> + 'a>>>) -> Result<Arc<ModuleInstance<E>>, InterpreterError<E>> {
		let mut module_instance = ModuleInstance::new(Arc::downgrade(&self.essence), name.into(), module)?;
		module_instance.instantiate(externals)?;

		let module_instance = Arc::new(module_instance);
		self.essence.modules.write().insert(name.into(), module_instance.clone());
		module_instance.run_start_function()?;
		Ok(module_instance)
	}

	/// Insert instantiated module.
	pub fn insert_loaded_module(&self, name: &str, module_instance: Arc<ModuleInstance<E>>) -> Result<Arc<ModuleInstance<E>>, Error> {
		// replace existing module with the same name with new one
		self.essence.modules.write().insert(name.into(), module_instance.clone());
		Ok(module_instance)
	}

	/// Get one of the modules by name
	pub fn module(&self, name: &str) -> Option<Arc<ModuleInstanceInterface<E>>> {
		self.essence.module(name)
	}
}

impl<E> ProgramInstanceEssence<E> where E: CustomUserError {
	/// Create new program essence.
	pub fn new() -> Result<Self, Error> {
		ProgramInstanceEssence::with_env_params(env::EnvParams::default())
	}

	pub fn with_env_params(env_params: env::EnvParams) -> Result<Self, Error> {
		let mut modules = HashMap::new();
		let env_module: Arc<ModuleInstanceInterface<E>> = Arc::new(env_module(env_params)?);
		modules.insert("env".into(), env_module);
		Ok(ProgramInstanceEssence {
			modules: RwLock::new(modules),
		})		
	}

	/// Get module reference.
	pub fn module(&self, name: &str) -> Option<Arc<ModuleInstanceInterface<E>>> {
		self.modules.read().get(name).cloned()
	}
}
