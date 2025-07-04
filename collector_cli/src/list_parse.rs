use collector_engine::parser::YamlArtifact;
use std::fmt;

pub struct ArtifactListing {
	pub artifact_struct_list: Vec<YamlArtifact>,
}

#[derive(Debug)]
pub struct ArtifactCategoryNames {
	category_name: String,
	list_name_artifact: Vec<String>,
}

impl fmt::Display for ArtifactCategoryNames {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
		let width: usize = 20;
		let n: usize = self.list_name_artifact.len();
		write!(f,"{:>width$} | {}\n",self.category_name,self.list_name_artifact[0])?;
		for number in 1..n {
			write!(f,"{:>width$} | {}\n","",self.list_name_artifact[number])?;
		}
		Ok(())
	}
}

impl ArtifactListing {
	pub fn load(artifact_struct: Vec<YamlArtifact>) -> Self {
		ArtifactListing {
			artifact_struct_list: artifact_struct,
		}
	}

	/// List all target has been found
	pub fn names_pa(&self) -> Vec<String>{
		let mut names: Vec<String> = Vec::new();
		for artifact in &self.artifact_struct_list{
			if artifact.artifact.path.is_some(){
				names.push(artifact.metadata.name.clone());
			}
		}
		names
	}

	/// List all group has been found
	pub fn names_gr(&self) -> Vec<String>{
		let mut names: Vec<String> = Vec::new();
		for artifact in &self.artifact_struct_list{
			if artifact.artifact.group.is_some(){
				names.push(artifact.metadata.name.clone());
			}
		}
		names
	}

	/// List all categories and is correspond group/target has be found.
	/// Categories is a string of categories
	/// and list in vector of groupt/target available in this categories
	pub fn list_categories(&self) -> Vec<ArtifactCategoryNames>{
		let mut names: Vec<ArtifactCategoryNames> = Vec::new();
		for artifact in &self.artifact_struct_list{
			if artifact.metadata.category.is_some(){
				let category = artifact.metadata.category.clone().unwrap();
				let finder = names.iter_mut().find(|e| e.category_name == category);
				if finder.is_some(){
					let choosen: &mut ArtifactCategoryNames = finder.unwrap();
					let _ = &choosen.list_name_artifact.push(artifact.metadata.name.clone());
				}else{
					let build = ArtifactCategoryNames{
						category_name: category,
						list_name_artifact: vec![artifact.metadata.name.clone()],
					};
					names.push(build);
				}
			}else{
				let finder = names.iter_mut().find(|e| e.category_name == "Other".to_string());
				if finder.is_some(){
					let choosen: &mut ArtifactCategoryNames = finder.unwrap();
					let _ = &choosen.list_name_artifact.push(artifact.metadata.name.clone());
				}else{
					let build = ArtifactCategoryNames{
						category_name: "Other".to_string(),
						list_name_artifact: vec![artifact.metadata.name.clone()],
					};
					names.push(build);
				}
			}
		}
		names
	}
}