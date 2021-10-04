use crate::shader::{
    ComputeShader, DomainShader, GeometryShader, HullShader, PixelShader, Shader, ShaderPointer,
    ShaderType, VertexShader,
};
use notify::{Op, RawEvent, RecommendedWatcher, RecursiveMode, Watcher};
use path_abs::{PathDir, PathFile};
use pathdiff::diff_paths;
use slotmap::{DefaultKey, SlotMap};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::{fs, io};
use winapi::um::d3d11::{
    ID3D11ComputeShader, ID3D11Device, ID3D11DomainShader, ID3D11GeometryShader, ID3D11HullShader,
    ID3D11PixelShader, ID3D11VertexShader,
};

pub trait DependencyGetter {
    fn get_content(&mut self, absolute_path: PathFile) -> io::Result<String>;
}

struct ManagerDependencyGetter<'a> {
    shader_key: DefaultKey,
    depends_on: Vec<PathFile>,
    dependencies: &'a mut HashMap<PathFile, ShaderDependency>,
}

impl<'a> DependencyGetter for ManagerDependencyGetter<'a> {
    fn get_content(&mut self, absolute_path: PathFile) -> io::Result<String> {
        let dependency = match self.dependencies.entry(absolute_path.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(ShaderDependency {
                depended_by: HashSet::new(),
                content_cache: fs::read_to_string(&absolute_path)?,
            }),
        };

        self.depends_on.push(absolute_path);

        dependency.depended_by.insert(self.shader_key);
        Ok(dependency.content_cache.clone())
    }
}

struct ShaderDependency {
    depended_by: HashSet<DefaultKey>,
    content_cache: String,
}

pub enum GenericShader {
    Vertex(VertexShader),
    Geometry(GeometryShader),
    Pixel(PixelShader),
    Hull(HullShader),
    Domain(DomainShader),
    Compute(ComputeShader),
}

impl GenericShader {
    pub fn new<T: ShaderPointer>(
        device: *mut ID3D11Device,
        full_path: PathFile,
        dependency_getter: &mut DependencyGetter,
    ) -> Self {
        match T::shader_type() {
            ShaderType::Vertex => {
                GenericShader::Vertex(VertexShader::new(device, full_path, dependency_getter))
            }
            ShaderType::Geometry => {
                GenericShader::Geometry(GeometryShader::new(device, full_path, dependency_getter))
            }
            ShaderType::Pixel => {
                GenericShader::Pixel(PixelShader::new(device, full_path, dependency_getter))
            }
            ShaderType::Hull => {
                GenericShader::Hull(HullShader::new(device, full_path, dependency_getter))
            }
            ShaderType::Domain => {
                GenericShader::Domain(DomainShader::new(device, full_path, dependency_getter))
            }
            ShaderType::Compute => {
                GenericShader::Compute(ComputeShader::new(device, full_path, dependency_getter))
            }
        }
    }

    pub fn rebuild(&mut self, device: *mut ID3D11Device, dependency_getter: &mut DependencyGetter) {
        match self {
            GenericShader::Vertex(shader) => shader.rebuild(device, dependency_getter),
            GenericShader::Geometry(shader) => shader.rebuild(device, dependency_getter),
            GenericShader::Pixel(shader) => shader.rebuild(device, dependency_getter),
            GenericShader::Hull(shader) => shader.rebuild(device, dependency_getter),
            GenericShader::Domain(shader) => shader.rebuild(device, dependency_getter),
            GenericShader::Compute(shader) => shader.rebuild(device, dependency_getter),
        }
    }

    pub fn as_vertex(&self) -> Option<&VertexShader> {
        match self {
            GenericShader::Vertex(shader) => Some(shader),
            _ => None,
        }
    }

    pub fn as_geometry(&self) -> Option<&GeometryShader> {
        match self {
            GenericShader::Geometry(shader) => Some(shader),
            _ => None,
        }
    }

    pub fn as_pixel(&self) -> Option<&PixelShader> {
        match self {
            GenericShader::Pixel(shader) => Some(shader),
            _ => None,
        }
    }

    pub fn as_hull(&self) -> Option<&HullShader> {
        match self {
            GenericShader::Hull(shader) => Some(shader),
            _ => None,
        }
    }

    pub fn as_domain(&self) -> Option<&DomainShader> {
        match self {
            GenericShader::Domain(shader) => Some(shader),
            _ => None,
        }
    }

    pub fn as_compute(&self) -> Option<&ComputeShader> {
        match self {
            GenericShader::Compute(shader) => Some(shader),
            _ => None,
        }
    }

    pub fn as_ref<T: ShaderPointer>(&self) -> Option<&Shader<T>> {
        T::unwrap_generic(self)
    }
}

struct RootShader {
    shader: GenericShader,
    depends_on: Vec<PathFile>,
}

pub struct ShaderKey<T: ShaderPointer> {
    key: DefaultKey,
    _phantom: PhantomData<*mut T>,
}

impl<T: ShaderPointer> Clone for ShaderKey<T> {
    fn clone(&self) -> Self {
        ShaderKey {
            key: self.key,
            _phantom: PhantomData,
        }
    }
}

impl<T: ShaderPointer> Copy for ShaderKey<T> {}

pub type VertexKey = ShaderKey<ID3D11VertexShader>;
pub type GeometryKey = ShaderKey<ID3D11GeometryShader>;
pub type PixelKey = ShaderKey<ID3D11PixelShader>;
pub type HullKey = ShaderKey<ID3D11HullShader>;
pub type DomainKey = ShaderKey<ID3D11DomainShader>;
pub type ComputeKey = ShaderKey<ID3D11ComputeShader>;

#[derive(Hash, PartialEq, Eq)]
struct ShaderPathKey {
    path: PathFile,
    shader_type: ShaderType,
}

pub struct ShaderManager<'shaders> {
    root: PathDir,

    _watcher: RecommendedWatcher,
    rx: Receiver<RawEvent>,

    shader_paths: HashMap<ShaderPathKey, DefaultKey>,
    shaders: SlotMap<DefaultKey, RootShader>,
    dependencies: HashMap<PathFile, ShaderDependency>,

    path_journal: Vec<(ShaderType, PathFile)>,

    _phantom: PhantomData<&'shaders [c_void]>,
}

impl ShaderManager<'_> {
    pub fn new(root_path: PathDir) -> Self {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new_raw(tx).unwrap();

        watcher.watch(&root_path, RecursiveMode::Recursive).unwrap();

        ShaderManager {
            root: root_path,
            _watcher: watcher,
            rx,
            shader_paths: HashMap::new(),
            shaders: SlotMap::new(),
            dependencies: HashMap::new(),
            path_journal: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn path_journal(&self) -> &[(ShaderType, PathFile)] {
        &self.path_journal
    }

    pub fn load_shader<T: ShaderPointer>(
        &mut self,
        device: *mut ID3D11Device,
        path: &str,
    ) -> ShaderKey<T> {
        let fixed_path = path.replace("/", "\\");

        let full_path = self
            .root
            .join(&fixed_path)
            .absolute()
            .unwrap()
            .into_file()
            .unwrap();

        self.path_journal
            .push((T::shader_type(), full_path.clone()));

        let shaders = &mut self.shaders;
        let dependencies = &mut self.dependencies;

        let shader_path_key = ShaderPathKey {
            path: full_path.clone(),
            shader_type: T::shader_type(),
        };
        let shader_key = *self.shader_paths.entry(shader_path_key).or_insert_with(|| {
            let shader_key = shaders.insert_with_key(|key| {
                let mut dependency_getter = ManagerDependencyGetter {
                    shader_key: key,
                    depends_on: Vec::new(),
                    dependencies,
                };

                RootShader {
                    shader: GenericShader::new::<T>(device, full_path, &mut dependency_getter),
                    depends_on: Vec::new(),
                }
            });

            shader_key
        });

        ShaderKey {
            key: shader_key,
            _phantom: PhantomData,
        }
    }

    pub fn get_shader<T: ShaderPointer>(&self, key: ShaderKey<T>) -> Option<&Shader<T>> {
        self.shaders
            .get(key.key)
            .map(|root_shader| root_shader.shader.as_ref().unwrap())
    }

    pub fn update(&mut self, device: *mut ID3D11Device) {
        // process all write events that have been received since the last update
        let change_events = self.rx.try_iter().filter(|evt| {
            evt.op
                .as_ref()
                .map(|op| op.contains(Op::WRITE))
                .unwrap_or(false)
        });
        let mut update_paths: Vec<_> = change_events
            .map(|change_event| PathFile::new(change_event.path.unwrap()).unwrap())
            .collect();

        // hot path: skip everything if there are no changes
        if update_paths.is_empty() {
            return;
        }

        update_paths.sort();
        update_paths.dedup();

        // build a list of affected shaders
        let mut updated_shaders: Vec<_> = update_paths
            .iter()
            .filter_map(|update_path| {
                let dependency = self.dependencies.get(update_path);

                if let None = dependency {
                    println!(
                        "\"{}\" changed but isn't used, ignoring",
                        diff_paths(update_path, &self.root)
                            .unwrap()
                            .to_str()
                            .unwrap()
                    );
                }

                dependency
            })
            .flat_map(|dependency| dependency.depended_by.iter().map(|&key| key))
            .collect();
        updated_shaders.sort();
        updated_shaders.dedup();

        // remove the invalidated shader dependencies
        for updated_path in update_paths.into_iter() {
            self.dependencies.remove(&updated_path);
        }

        // update shaders and dependencies
        let mut dependency_gc_candidates = HashSet::new();
        for updated_shader_key in updated_shaders.into_iter() {
            let updated_shader = self.shaders.get_mut(updated_shader_key).unwrap();

            // remove the shader from any dependencies
            for depends_on_path in &updated_shader.depends_on {
                if let Some(dependency) = self.dependencies.get_mut(depends_on_path) {
                    dependency.depended_by.remove(&updated_shader_key);

                    if dependency.depended_by.is_empty() {
                        dependency_gc_candidates.insert(depends_on_path.clone());
                    }
                }
            }

            let mut dependency_getter = ManagerDependencyGetter {
                shader_key: updated_shader_key,
                depends_on: Vec::new(),
                dependencies: &mut self.dependencies,
            };
            updated_shader
                .shader
                .rebuild(device, &mut dependency_getter);
            updated_shader.depends_on = dependency_getter.depends_on;
        }

        // garbage-collect any dependencies that don't have references
        for gc_candidate in dependency_gc_candidates.into_iter() {
            if self.dependencies[&gc_candidate].depended_by.is_empty() {
                self.dependencies.remove(&gc_candidate);
            }
        }
    }
}

impl<T: ShaderPointer> ops::Index<ShaderKey<T>> for ShaderManager<'_> {
    type Output = Shader<T>;

    fn index(&self, key: ShaderKey<T>) -> &Shader<T> {
        self.get_shader(key).unwrap()
    }
}
