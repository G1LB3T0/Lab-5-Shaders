use nalgebra_glm as glm;

pub struct FlowmapTexture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>, // RGBA
}

impl FlowmapTexture {
    pub fn sample(&self, u: f32, v: f32) -> (f32, f32, f32) {
        let u_wrapped = u - u.floor();
        let v_wrapped = v - v.floor();
        
        let x = (u_wrapped * self.width as f32) as usize % self.width;
        let y = (v_wrapped * self.height as f32) as usize % self.height;
        
        let idx = (y * self.width + x) * 4;
        let r = self.data[idx] as f32 / 255.0;
        let g = self.data[idx + 1] as f32 / 255.0;
        let b = self.data[idx + 2] as f32 / 255.0;
        
        (r, g, b)
    }
    
    pub fn sample_flow(&self, u: f32, v: f32) -> (f32, f32) {
        let (r, g, _) = self.sample(u, v);
        // Convertir de [0,1] a [-1,1] para vectores de flujo
        let flow_x = r * 2.0 - 1.0;
        let flow_y = g * 2.0 - 1.0;
        (flow_x, flow_y)
    }
}

pub struct Uniforms<'a> {
    pub base_color: (u8,u8,u8),
    pub light_dir: glm::Vec3,
    pub ambient: f32,
    pub spec_power: f32,
    pub spec_strength: f32,
    pub rim_strength: f32,
    pub time: f32,
    pub flowmap: Option<&'a FlowmapTexture>,
}

pub struct TriInput {
    pub p0: glm::Vec3,
    pub p1: glm::Vec3,
    pub p2: glm::Vec3,
    pub n0: glm::Vec3,  // Normal suave del vértice 0
    pub n1: glm::Vec3,  // Normal suave del vértice 1
    pub n2: glm::Vec3,  // Normal suave del vértice 2
}

pub trait Shader {
    fn shade(&self, u:&Uniforms, tri:&TriInput) -> (u8,u8,u8);
}

// ============ SHADER MEJORADO: METAL ALIENÍGENA AVANZADO ============
// Metal futurista con textura procedural, anisotropía y efectos especiales
pub struct MetalLambert;

fn clamp01(x:f32)->f32 { x.max(0.0).min(1.0) }

impl Shader for MetalLambert {
    fn shade(&self, u:&Uniforms, tri:&TriInput) -> (u8,u8,u8) {
        let n_raw = (tri.p1 - tri.p0).cross(&(tri.p2 - tri.p0));
        let n = if n_raw.magnitude() > 1e-9 { n_raw.normalize() } else { glm::vec3(0.0,0.0,1.0) };

        let l = -u.light_dir.normalize();
        let v = glm::vec3(0.0, 0.0, 1.0); // cámara mira -Z

        // === ILUMINACIÓN DIFUSA ===
        let ndotl = clamp01(n.dot(&l));

        // === PATRÓN DE PLACAS METÁLICAS (Textura procedural) ===
        let center = (tri.p0 + tri.p1 + tri.p2) / 3.0;
        
        // Patrón de paneles hexagonales/celdas
        let panel_scale = 8.0;
        let panel_x = (center.x * panel_scale).sin();
        let panel_y = (center.y * panel_scale).cos();
        let panel_z = (center.z * panel_scale * 1.3).sin();
        let panel_pattern = ((panel_x + panel_y + panel_z) * 0.33).abs();
        
        // Líneas de desgaste/rayones metálicos
        let scratch_scale = 25.0;
        let scratches = ((center.x * scratch_scale).sin() * (center.y * scratch_scale * 0.7).cos()).abs();
        let scratch_effect = if scratches > 0.92 { 0.85 } else { 1.0 };
        
        // Variación de brillo por panel (diferentes acabados metálicos)
        let panel_brightness = 0.85 + panel_pattern * 0.3;

        // === EFECTO RIM LIGHT (Borde brillante tipo sci-fi) ===
        let ndotv = clamp01(n.dot(&v));
        let rim = (1.0 - ndotv).powf(2.5) * u.rim_strength * 1.8;
        
        // === SPECULAR HIGHLIGHTS (Reflexión brillante) ===
        let h = (l + v).normalize();
        let spec = if ndotl > 0.0 {
            // Specular principal (highlight fuerte)
            let main_spec = u.spec_strength * clamp01(n.dot(&h)).powf(u.spec_power);
            
            // Specular secundario anisótropo (simula metal pulido con dirección)
            let aniso_dir = glm::vec3(1.0, 0.0, 0.0); // dirección del pulido
            let ht = h - aniso_dir * aniso_dir.dot(&h);
            let aniso_spec = if ht.magnitude() > 0.001 {
                0.3 * clamp01(n.dot(&ht.normalize())).powf(15.0)
            } else {
                0.0
            };
            
            main_spec + aniso_spec
        } else { 
            0.0 
        };

        // === EFECTO DE METALICIDAD (reflejo ambiental simulado) ===
        // Los metales reflejan más el ambiente en ángulos rasantes
        let metallic_env = (1.0 - ndotv).powf(1.5) * 0.15;

        // === COMPOSICIÓN FINAL ===
        let diffuse = u.ambient + (1.0 - u.ambient) * ndotl * 0.7;
        
        // Intensidad base del material
        let base_intensity = diffuse * panel_brightness * scratch_effect;
        
        // Reflexiones metálicas (mantener balance)
        let reflections = rim * 0.4 + spec * 0.8 + metallic_env * 0.3;
        
        let final_intensity = clamp01(base_intensity + reflections);

        // Color metálico preservando el tinte base
        let (base_r, base_g, base_b) = u.base_color;
        
        let r = (base_r as f32 / 255.0 * final_intensity).clamp(0.0, 1.0) * 255.0;
        let g = (base_g as f32 / 255.0 * final_intensity).clamp(0.0, 1.0) * 255.0;
        let b = (base_b as f32 / 255.0 * final_intensity).clamp(0.0, 1.0) * 255.0;

        (r as u8, g as u8, b as u8)
    }
}

// ============ SHADER SOL: FLOWMAP PLASMA ============
pub struct SunShader;

// Función de smoothstep para transiciones suaves
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp01((x - edge0) / (edge1 - edge0));
    t * t * (3.0 - 2.0 * t)
}

// Ruido procedural simple
fn noise(x: f32, y: f32) -> f32 {
    let n = (x * 12.9898 + y * 78.233).sin() * 43758.5453;
    (n - n.floor()) * 2.0 - 1.0
}

impl Shader for SunShader {
    fn shade(&self, u:&Uniforms, tri:&TriInput) -> (u8,u8,u8) {
        // Normal SUAVE interpolada (Phong shading)
        let n_interpolated = (tri.n0 + tri.n1 + tri.n2) / 3.0;
        let n = n_interpolated.normalize();
        
        let v = glm::vec3(0.0, 0.0, 1.0);
        let ndotv = clamp01(n.dot(&v));
        
        let center = (tri.p0 + tri.p1 + tri.p2) / 3.0;
        let pos_normalized = center.normalize();
        
        // Convertir a coordenadas UV esféricas
        let theta = pos_normalized.y.asin();
        let phi = pos_normalized.z.atan2(pos_normalized.x);
        let uv_x = phi / (2.0 * std::f32::consts::PI) + 0.5;
        let uv_y = theta / std::f32::consts::PI + 0.5;
        
        // FLOWMAP: Generar campo de flujo procedural
        let flow_scale = 3.0;
        let flow_x = noise(uv_x * flow_scale, uv_y * flow_scale) * 0.5 + 
                     noise(uv_x * flow_scale * 2.0, uv_y * flow_scale * 2.0) * 0.25;
        let flow_y = noise(uv_x * flow_scale + 100.0, uv_y * flow_scale + 100.0) * 0.5 +
                     noise(uv_x * flow_scale * 2.0 + 100.0, uv_y * flow_scale * 2.0 + 100.0) * 0.25;
        
        // Animar flowmap con ciclo para evitar saltos
        let flow_cycle = 0.5;
        let phase0 = (u.time * 0.4) % flow_cycle;
        let phase1 = (u.time * 0.4 + flow_cycle * 0.5) % flow_cycle;
        let blend = (phase0 / flow_cycle).abs() * 2.0;
        let blend_smooth = smoothstep(0.0, 1.0, blend);
        
        // Aplicar flowmap para distorsionar UVs
        let flow_strength = 0.15;
        let uv0_x = uv_x + flow_x * phase0 * flow_strength;
        let uv0_y = uv_y + flow_y * phase0 * flow_strength;
        let uv1_x = uv_x + flow_x * phase1 * flow_strength;
        let uv1_y = uv_y + flow_y * phase1 * flow_strength;
        
        // Muestrear plasma con flowmap
        let plasma0 = (
            (uv0_x * 8.0).sin() * (uv0_y * 8.0).cos() +
            (uv0_x * 15.0).cos() * (uv0_y * 15.0).sin() * 0.5
        ) * 0.5 + 0.5;
        
        let plasma1 = (
            (uv1_x * 8.0).sin() * (uv1_y * 8.0).cos() +
            (uv1_x * 15.0).cos() * (uv1_y * 15.0).sin() * 0.5
        ) * 0.5 + 0.5;
        
        let plasma = plasma0 * (1.0 - blend_smooth) + plasma1 * blend_smooth;
        let plasma_smooth = smoothstep(0.3, 0.7, plasma);
        
        // Manchas solares
        let sunspot = ((uv_x * 12.0 + u.time * 0.1).sin() * (uv_y * 12.0).cos() + 1.0) * 0.5;
        let sunspot_smooth = smoothstep(0.4, 0.6, sunspot);
        let darkening = 0.78 + sunspot_smooth * 0.22;
        
        // Corona brillante y pulsante
        let corona_pulse = 1.0 + (u.time * 1.8).sin() * 0.1;
        let corona = (1.0 - ndotv).powf(2.3) * 0.6 * corona_pulse;
        
        // Iluminación esférica
        let sphere_lighting = smoothstep(0.0, 1.0, ndotv * 0.5 + 0.5);
        
        // Intensidad brillante
        let base_intensity = 0.9 + plasma_smooth * 0.2;
        let intensity = clamp01(base_intensity * darkening * sphere_lighting + corona * 1.4);
        
        // Colores amarillo-blanco brillantes
        let r = clamp01(intensity * 1.08 + corona * 0.3) * 255.0;
        let g = clamp01(intensity * 0.98 + corona * 0.25) * 255.0;
        let b = clamp01(intensity * 0.68 + corona * 0.15) * 255.0;
        
        (r as u8, g as u8, b as u8)
    }
}

// ============ SHADER PLANETA ROCOSO ============
pub struct RockyPlanetShader;

impl Shader for RockyPlanetShader {
    fn shade(&self, u:&Uniforms, tri:&TriInput) -> (u8,u8,u8) {
        // Normal SUAVE interpolada (Phong shading)
        let n_interpolated = (tri.n0 + tri.n1 + tri.n2) / 3.0;
        let n = n_interpolated.normalize();
        
        let l = -u.light_dir.normalize();
        let v = glm::vec3(0.0, 0.0, 1.0);
        
        let ndotl = clamp01(n.dot(&l));
        
        let center = (tri.p0 + tri.p1 + tri.p2) / 3.0;
        let pos_normalized = center.normalize();
        
        // Terreno con múltiples octavas suavizadas
        let terrain1 = ((pos_normalized.x * 3.5).sin() * (pos_normalized.y * 3.5).cos() + (pos_normalized.z * 3.5).sin()) * 0.35;
        let terrain2 = ((pos_normalized.x * 7.5).sin() * (pos_normalized.z * 7.5).cos()) * 0.2;
        let terrain3 = ((pos_normalized.y * 13.0).cos() * (pos_normalized.z * 13.0).sin()) * 0.12;
        let terrain_raw = (terrain1 + terrain2 + terrain3 + 1.0) * 0.5;
        let terrain = smoothstep(0.2, 0.8, terrain_raw);
        
        // Cráteres con transición suave
        let crater1_raw = ((pos_normalized.x * 7.0).cos() * (pos_normalized.y * 7.0).sin() + 1.0) * 0.5;
        let crater2_raw = ((pos_normalized.z * 10.0).sin() * (pos_normalized.x * 10.0).cos() + 1.0) * 0.5;
        let crater1 = smoothstep(0.35, 0.65, crater1_raw);
        let crater2 = smoothstep(0.4, 0.6, crater2_raw);
        let crater = crater1 * 0.6 + crater2 * 0.4;
        let crater_effect = 0.72 + crater * 0.28;
        
        // Casquetes polares con fade muy suave
        let pole_dist = pos_normalized.y.abs();
        let pole_fade = smoothstep(0.55, 0.85, pole_dist);
        let polar = 1.0 + pole_fade * 0.45;
        
        // Iluminación difusa suave con wrap-around
        let ndotl_wrapped = (ndotl + 0.3) / 1.3;
        let diffuse = u.ambient + (1.0 - u.ambient) * smoothstep(0.0, 1.0, ndotl_wrapped);
        
        // Especular sutil
        let h = (l + v).normalize();
        let spec = if ndotl > 0.0 {
            0.06 * smoothstep(0.0, 1.0, n.dot(&h)).powf(28.0)
        } else {
            0.0
        };
        
        let intensity = clamp01(diffuse * crater_effect * polar + spec);
        
        // Colores rocosos con variación suave
        let base_r = 135.0 + terrain * 55.0;
        let base_g = 68.0 + terrain * 48.0 + crater * 22.0;
        let base_b = 32.0 + terrain * 32.0;
        
        let r = clamp01((base_r / 255.0) * intensity) * 255.0;
        let g = clamp01((base_g / 255.0) * intensity) * 255.0;
        let b = clamp01((base_b / 255.0) * intensity) * 255.0;
        
        (r as u8, g as u8, b as u8)
    }
}

// ============ SHADER GIGANTE GASEOSO: FLOWMAP PARA DISTORSIÓN UV ============
pub struct GasGiantShader;

impl Shader for GasGiantShader {
    fn shade(&self, u:&Uniforms, tri:&TriInput) -> (u8,u8,u8) {
        // Interpolación baricéntrica de normales SUAVES (simulando fragment shader)
        let n_interpolated = (tri.n0 + tri.n1 + tri.n2) / 3.0;
        let n = n_interpolated.normalize(); // Normal SUAVE interpolada
        
        let l = -u.light_dir.normalize();
        let v = glm::vec3(0.0, 0.0, 1.0);
        
        let ndotl = clamp01(n.dot(&l));
        let ndotv = clamp01(n.dot(&v));
        
        let center = (tri.p0 + tri.p1 + tri.p2) / 3.0;
        let pos_normalized = center.normalize();
        
        // Coordenadas UV esféricas BASE
        let theta = pos_normalized.y.asin();
        let phi = pos_normalized.z.atan2(pos_normalized.x);
        let base_uv_x = phi / (2.0 * std::f32::consts::PI) + 0.5;
        let base_uv_y = theta / std::f32::consts::PI + 0.5;
        
        // Calcular UVs distorsionados usando el flowmap
        let (distorted_uv_x, distorted_uv_y) = if let Some(flowmap) = u.flowmap {
            // Leer VECTORES DE FLUJO desde el flowmap (R=U, G=V)
            let (flow_u, flow_v) = flowmap.sample_flow(base_uv_x, base_uv_y);
            
            // Ciclo temporal para animación continua sin saltos
            let flow_cycle = 1.5;
            let phase0 = (u.time * 0.25) % flow_cycle;
            let phase1 = (u.time * 0.25 + flow_cycle * 0.5) % flow_cycle;
            let blend_factor = (phase0 / flow_cycle) * 2.0;
            let blend = smoothstep(0.0, 1.0, blend_factor.min(1.0));
            
            // Aplicar distorsión del flowmap con dos fases
            let flow_strength = 0.2;
            let uv0_x = base_uv_x + flow_u * phase0 * flow_strength;
            let uv0_y = base_uv_y + flow_v * phase0 * flow_strength;
            let uv1_x = base_uv_x + flow_u * phase1 * flow_strength;
            let uv1_y = base_uv_y + flow_v * phase1 * flow_strength;
            
            // Interpolar entre las dos fases
            let final_u = uv0_x * (1.0 - blend) + uv1_x * blend;
            let final_v = uv0_y * (1.0 - blend) + uv1_y * blend;
            
            (final_u, final_v)
        } else {
            (base_uv_x, base_uv_y)
        };
        
        // TEXTURA PROCEDURAL DE GAS/NUBES usando UVs distorsionados
        // Múltiples octavas de ruido para atmósfera realista
        let cloud_layer1 = ((distorted_uv_x * 8.0).sin() * (distorted_uv_y * 8.0).cos() + 
                           (distorted_uv_y * 8.0).sin()) * 0.5 + 0.5;
        let cloud_layer2 = ((distorted_uv_x * 16.0 + 0.5).cos() * (distorted_uv_y * 16.0 - 0.3).sin() + 1.0) * 0.5;
        let cloud_layer3 = ((distorted_uv_x * 32.0 + 1.0).sin() * (distorted_uv_y * 32.0 + 0.7).cos() + 1.0) * 0.5;
        
        let cloud_detail = cloud_layer1 * 0.5 + cloud_layer2 * 0.3 + cloud_layer3 * 0.2;
        let cloud_smooth = smoothstep(0.3, 0.7, cloud_detail);
        
        // Bandas atmosféricas (latitud) con las UVs distorsionadas
        let latitude_bands = ((distorted_uv_y * 12.0).sin() + 1.0) * 0.5;
        let band_smooth = smoothstep(0.35, 0.65, latitude_bands);
        
        // Turbulencia fina
        let turbulence = ((distorted_uv_x * 40.0 + distorted_uv_y * 20.0).sin() + 1.0) * 0.5;
        let turb_smooth = smoothstep(0.4, 0.6, turbulence);
        
        // Gran Mancha Roja (estática en UV space)
        let storm_center_u = 0.7;
        let storm_center_v = 0.35;
        let du = distorted_uv_x - storm_center_u;
        let dv = (distorted_uv_y - storm_center_v) * 1.8;
        let storm_dist = (du * du + dv * dv).sqrt();
        let storm_size = 0.12;
        let storm = smoothstep(storm_size, 0.0, storm_dist);
        
        // COLORES DE GAS (Júpiter-like)
        let light_color = (210.0 + turb_smooth * 25.0, 175.0 + turb_smooth * 30.0, 130.0 + turb_smooth * 20.0);
        let dark_color = (150.0 + turb_smooth * 20.0, 110.0 + turb_smooth * 20.0, 70.0 + turb_smooth * 15.0);
        
        // Mezclar bandas con nubes
        let base_r = light_color.0 * band_smooth + dark_color.0 * (1.0 - band_smooth);
        let base_g = light_color.1 * band_smooth + dark_color.1 * (1.0 - band_smooth);
        let base_b = light_color.2 * band_smooth + dark_color.2 * (1.0 - band_smooth);
        
        // Aplicar detalle de nubes
        let cloud_r = base_r * (0.85 + cloud_smooth * 0.3);
        let cloud_g = base_g * (0.85 + cloud_smooth * 0.3);
        let cloud_b = base_b * (0.85 + cloud_smooth * 0.3);
        
        // Mezclar con tormenta roja
        let storm_color = (200.0, 80.0, 60.0);
        let final_r = cloud_r * (1.0 - storm) + storm_color.0 * storm;
        let final_g = cloud_g * (1.0 - storm) + storm_color.1 * storm;
        let final_b = cloud_b * (1.0 - storm) + storm_color.2 * storm;
        
        // Iluminación volumétrica
        let ndotl_wrapped = (ndotl + 0.4) / 1.4;
        let diffuse = u.ambient + (1.0 - u.ambient) * smoothstep(0.0, 1.0, ndotl_wrapped * 0.9);
        
        // Atmósfera en bordes
        let atmosphere = smoothstep(0.0, 1.0, 1.0 - ndotv).powf(2.0) * 0.25;
        
        let intensity = clamp01(diffuse + atmosphere);
        
        let r = clamp01((final_r / 255.0) * intensity) * 255.0;
        let g = clamp01((final_g / 255.0) * intensity) * 255.0;
        let b = clamp01((final_b / 255.0) * intensity) * 255.0;
        
        (r as u8, g as u8, b as u8)
    }
}
