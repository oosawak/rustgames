// 数学モジュール: 行列演算、カメラ計算、乱数生成器を定義する

// 4×4 行列型（列優先）
pub type M4 = [[f32; 4]; 4];

pub fn mat_mul(a: M4, b: M4) -> M4 {
    let mut r = [[0f32; 4]; 4];
    for c in 0..4 { for row in 0..4 {
        r[c][row] = (0..4).map(|k| a[k][row] * b[c][k]).sum();
    }}
    r
}

pub fn perspective(fov: f32, asp: f32, n: f32, f: f32) -> M4 {
    let t = 1.0 / (fov * 0.5).tan();
    [[t/asp,0.0,0.0,0.0],[0.0,t,0.0,0.0],
     [0.0,0.0,f/(n-f),-1.0],[0.0,0.0,n*f/(n-f),0.0]]
}

pub fn norm3(v:[f32;3])->[f32;3]{let l=(v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt();if l<1e-7{[0.0,0.0,1.0]}else{[v[0]/l,v[1]/l,v[2]/l]}}
pub fn sub3(a:[f32;3],b:[f32;3])->[f32;3]{[a[0]-b[0],a[1]-b[1],a[2]-b[2]]}
pub fn cross(a:[f32;3],b:[f32;3])->[f32;3]{[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]}
pub fn dot3(a:[f32;3],b:[f32;3])->f32{a[0]*b[0]+a[1]*b[1]+a[2]*b[2]}

pub fn look_at(eye:[f32;3],ctr:[f32;3],up:[f32;3])->M4{
    let f=norm3(sub3(ctr,eye));let r=norm3(cross(f,norm3(up)));let u=cross(r,f);
    [[r[0],u[0],-f[0],0.0],[r[1],u[1],-f[1],0.0],[r[2],u[2],-f[2],0.0],
     [-dot3(r,eye),-dot3(u,eye),dot3(f,eye),1.0]]
}

// LCG 疑似乱数（整数）
pub fn lcg(s: &mut u64) -> usize {
    *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*s >> 33) as usize
}

// LCG 疑似乱数（0.0〜1.0 の浮動小数点）
pub fn lcg_f(s: &mut u64) -> f32 { lcg(s) as f32 / (u32::MAX as f32) }
