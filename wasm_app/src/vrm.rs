use serde::Serialize;
use std::convert::TryFrom;
use serde_json::Value;
use wasm_bindgen::prelude::*;

const GLB_MAGIC: u32 = 0x4654_6c67;
const JSON_CHUNK: u32 = 0x4e4f_534a;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VrmSourceFormat {
    GltfJson,
    Glb,
}

#[derive(Debug, Clone, Serialize)]
pub struct VrmHumanBone {
    pub bone: String,
    pub node: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct VrmInspection {
    pub source_format: VrmSourceFormat,
    pub asset_version: Option<String>,
    pub generator: Option<String>,
    pub node_count: usize,
    pub mesh_count: usize,
    pub skin_count: usize,
    pub material_count: usize,
    pub extensions_used: Vec<String>,
    pub extensions_required: Vec<String>,
    pub vrm_extension_name: Option<String>,
    pub vrm_version: Option<String>,
    pub vrm_title: Option<String>,
    pub vrm_author: Option<String>,
    pub human_bones: Vec<VrmHumanBone>,
    pub blendshape_group_count: usize,
}

impl VrmInspection {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let (source_format, json_bytes) = extract_gltf_json(bytes)?;
        let root: Value = serde_json::from_slice(&json_bytes)
            .map_err(|e| format!("failed to parse glTF JSON: {e}"))?;

        let asset_version = root
            .pointer("/asset/version")
            .and_then(Value::as_str)
            .map(str::to_string);
        let generator = root
            .pointer("/asset/generator")
            .and_then(Value::as_str)
            .map(str::to_string);
        let node_count = array_len(&root, "nodes");
        let mesh_count = array_len(&root, "meshes");
        let skin_count = array_len(&root, "skins");
        let material_count = array_len(&root, "materials");
        let extensions_used = string_array(&root, "extensionsUsed");
        let extensions_required = string_array(&root, "extensionsRequired");

        let vrm_extension = root
            .pointer("/extensions/VRM")
            .or_else(|| root.pointer("/extensions/VRMC_vrm"));
        let vrm_extension_name = vrm_extension.map(|_| {
            if root.pointer("/extensions/VRMC_vrm").is_some() {
                "VRMC_vrm".to_string()
            } else {
                "VRM".to_string()
            }
        });

        let vrm_version = vrm_extension
            .and_then(|ext| ext.get("specVersion").or_else(|| ext.get("version")))
            .and_then(Value::as_str)
            .map(str::to_string);

        let vrm_title = vrm_extension
            .and_then(|ext| ext.get("meta"))
            .and_then(|meta| meta.get("title"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let vrm_author = vrm_extension
            .and_then(|ext| ext.get("meta"))
            .and_then(|meta| meta.get("author").or_else(|| meta.get("authors")))
            .and_then(parse_author_name);

        let human_bones = vrm_extension
            .and_then(|ext| ext.get("humanoid"))
            .and_then(|humanoid| humanoid.get("humanBones").or_else(|| humanoid.get("human_bones")))
            .map(parse_human_bones)
            .unwrap_or_default();

        let blendshape_group_count = vrm_extension
            .and_then(|ext| {
                ext.get("blendShapeMaster")
                    .and_then(|m| m.get("blendShapeGroups"))
                    .or_else(|| ext.get("expressions"))
                    .or_else(|| ext.get("blendshapeGroups"))
            })
            .map(array_count_or_object_len)
            .unwrap_or(0);

        Ok(Self {
            source_format,
            asset_version,
            generator,
            node_count,
            mesh_count,
            skin_count,
            material_count,
            extensions_used,
            extensions_required,
            vrm_extension_name,
            vrm_version,
            vrm_title,
            vrm_author,
            human_bones,
            blendshape_group_count,
        })
    }
}

#[wasm_bindgen]
pub fn inspect_vrm(data: Vec<u8>) -> String {
    match VrmInspection::from_bytes(&data) {
        Ok(inspection) => serde_json::to_string(&inspection)
            .unwrap_or_else(|e| serde_json::json!({"error": format!("failed to serialize VRM summary: {e}")}).to_string()),
        Err(e) => serde_json::json!({"error": e}).to_string(),
    }
}

fn extract_gltf_json(bytes: &[u8]) -> Result<(VrmSourceFormat, Vec<u8>), String> {
    if bytes.len() >= 12 && read_u32(bytes, 0)? == GLB_MAGIC {
        let total_len = read_u32(bytes, 8)? as usize;
        if total_len > bytes.len() {
            return Err("GLB length exceeds provided data".to_string());
        }

        let mut offset = 12usize;
        while offset + 8 <= total_len {
            let chunk_len = read_u32(bytes, offset)? as usize;
            let chunk_type = read_u32(bytes, offset + 4)?;
            offset += 8;

            let end = offset.saturating_add(chunk_len);
            if end > total_len {
                return Err("GLB chunk length exceeds provided data".to_string());
            }

            if chunk_type == JSON_CHUNK {
                return Ok((VrmSourceFormat::Glb, bytes[offset..end].to_vec()));
            }

            offset = align4(end);
        }

        return Err("GLB file does not contain a JSON chunk".to_string());
    }

    Ok((VrmSourceFormat::GltfJson, bytes.to_vec()))
}

fn parse_human_bones(value: &Value) -> Vec<VrmHumanBone> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                let bone = item
                    .get("bone")
                    .or_else(|| item.get("name"))
                    .and_then(Value::as_str)?
                    .to_string();
                let node = item
                    .get("node")
                    .or_else(|| item.get("nodeIndex"))
                    .and_then(as_u32)?;
                Some(VrmHumanBone { bone, node })
            })
            .collect(),
        Value::Object(map) => map
            .iter()
            .filter_map(|(bone, item)| {
                let node = if let Some(node) = item.get("node").and_then(as_u32) {
                    Some(node)
                } else if let Some(node) = item.get("nodeIndex").and_then(as_u32) {
                    Some(node)
                } else if item.is_u64() {
                    as_u32(item)
                } else {
                    None
                }?;
                Some(VrmHumanBone {
                    bone: bone.clone(),
                    node,
                })
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn parse_author_name(value: &Value) -> Option<String> {
    match value {
        Value::String(name) => Some(name.clone()),
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .next(),
        _ => None,
    }
}

fn string_array(root: &Value, key: &str) -> Vec<String> {
    root.get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn array_len(root: &Value, key: &str) -> usize {
    root.get(key)
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

fn array_count_or_object_len(value: &Value) -> usize {
    match value {
        Value::Array(items) => items.len(),
        Value::Object(map) => map.len(),
        _ => 0,
    }
}

fn as_u32(value: &Value) -> Option<u32> {
    value.as_u64().and_then(|n| u32::try_from(n).ok())
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let end = offset.checked_add(4).ok_or_else(|| "offset overflow".to_string())?;
    let chunk = bytes
        .get(offset..end)
        .ok_or_else(|| "unexpected end of data".to_string())?;
    Ok(u32::from_le_bytes(chunk.try_into().unwrap()))
}

fn align4(value: usize) -> usize {
    (value + 3) & !3
}
