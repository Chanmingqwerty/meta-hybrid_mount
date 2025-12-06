use std::{collections::HashMap, fs::FileType, path::PathBuf};

use crate::core::modules::ModuleFile;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NodeFileType {
    Directory,
    RegularFile,
    Symlink,
    Whiteout,
}

impl From<FileType> for NodeFileType {
    fn from(file_type: FileType) -> Self {
        if file_type.is_dir() {
            Self::Directory
        } else if file_type.is_symlink() {
            Self::Symlink
        } else {
            Self::RegularFile
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub file_type: NodeFileType,
    pub module_path: Option<PathBuf>,
    pub children: HashMap<String, Node>,
    pub replace: bool,
    pub skip: bool,
}

impl Node {
    pub fn new_root(name: &str) -> Self {
        Self {
            name: name.to_string(),
            file_type: NodeFileType::Directory,
            module_path: None,
            children: HashMap::new(),
            replace: false,
            skip: false,
        }
    }

    pub fn collect_module_files(&mut self, root: &PathBuf) -> anyhow::Result<()> {
        for entry in walkdir::WalkDir::new(root)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let relative_path = path.strip_prefix(root)?;
            let module_file = ModuleFile::new(root, relative_path)?;
            self.add_module_file(module_file);
        }
        Ok(())
    }

    fn add_module_file(&mut self, module_file: ModuleFile) {
        let mut current_node = self;
        let components: Vec<_> = module_file.relative_path.components().collect();

        for (i, component) in components.iter().enumerate() {
            let name = component.as_os_str().to_string_lossy().to_string();
            let is_last = i == components.len() - 1;

            if is_last {
                let file_type = if module_file.is_whiteout {
                    NodeFileType::Whiteout
                } else {
                    NodeFileType::from(module_file.file_type)
                };

                let node = current_node
                    .children
                    .entry(name.clone())
                    .or_insert_with(|| Node {
                        name: name.clone(),
                        file_type,
                        module_path: None,
                        children: HashMap::new(),
                        replace: false,
                        skip: false,
                    });

                if !module_file.is_whiteout {
                    node.module_path = Some(module_file.real_path.clone());
                    node.file_type = file_type;
                } else {
                    node.file_type = NodeFileType::Whiteout;
                }
                
                if module_file.is_replace {
                    node.replace = true;
                }
            } else {
                current_node = current_node
                    .children
                    .entry(name.clone())
                    .or_insert_with(|| Node {
                        name,
                        file_type: NodeFileType::Directory,
                        module_path: None,
                        children: HashMap::new(),
                        replace: false,
                        skip: false,
                    });
            }
        }
    }
}
