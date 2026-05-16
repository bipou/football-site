use aws_lc_rs::hmac::{Key, sign};
use aws_lc_rs::rand;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::utils::constant;

const CAPTCHA_EXPIRY: u64 = 300; // 5 分钟

// ── 微型随机工具（用 aws-lc-rs 替代 rand crate）─────────────────────────────

fn random_u32() -> u32 {
    let mut buf = [0u8; 4];
    rand::fill(&mut buf).expect("aws-lc-rs rand fill");
    u32::from_be_bytes(buf)
}

/// [min, max] 闭区间
fn gen_range_i32(min: i32, max: i32) -> i32 {
    let range = (max - min + 1) as u32;
    min + (random_u32() % range) as i32
}

/// [min, max) 浮点
fn gen_range_f64(min: f64, max: f64) -> f64 {
    let f = (random_u32() as f64) / (u32::MAX as f64);
    min + f * (max - min)
}

fn gen_bool() -> bool {
    random_u32() & 1 == 0
}

// ── Captcha 数据结构 ──────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Captcha {
    pub svg: String,  // SVG 标记文本
    pub token: String, // HMAC 签名令牌
}

struct Payload {
    answer: u8,
    exp: u64,
}

// ── 令牌签名 ──────────────────────────────────────────────────────────────────

fn sign_payload(p: &Payload) -> String {
    let hmac_key = Key::new(aws_lc_rs::hmac::HMAC_SHA256, constant::config().site_key.as_bytes());
    let msg = format!("{}:{}", p.answer, p.exp);
    let sig = sign(&hmac_key, msg.as_bytes());
    let sig_b64 = URL_SAFE_NO_PAD.encode(sig.as_ref());
    format!("{}:{}", p.exp, sig_b64)
    // token = "1715000000:xxxxx_sig_xxxxx"
}

/// 验证令牌，返回答案；失败返回 None
pub fn verify_token(token: &str, user_answer: &str) -> Option<u8> {
    let (exp_str, sig_b64) = token.split_once(':')?;
    let exp: u64 = exp_str.parse().ok()?;

    // 过期检查
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    if now > exp {
        return None;
    }

    let user_ans: u8 = user_answer.parse().ok()?;

    // 验证 HMAC
    let hmac_key = Key::new(aws_lc_rs::hmac::HMAC_SHA256, constant::config().site_key.as_bytes());
    let msg = format!("{}:{}", user_ans, exp);
    let expected_sig = URL_SAFE_NO_PAD.encode(sign(&hmac_key, msg.as_bytes()).as_ref());

    if sig_b64 == expected_sig {
        Some(user_ans)
    } else {
        None
    }
}

// ── SVG 生成 ──────────────────────────────────────────────────────────────────

pub fn generate_captcha() -> Captcha {
    // 随机操作数 (1~9)
    let a = gen_range_i32(1, 9) as u8;
    let b = gen_range_i32(1, 9) as u8;
    let op = if gen_bool() { '+' } else { '-' };
    let answer = match op {
        '+' => a + b,
        '-' => {
            // 确保结果为正
            if a >= b { a - b } else { b - a }
        }
        _ => unreachable!(),
    };
    // 修正显示的操作数顺序（被减数 >= 减数）
    let (left, right) = if op == '-' && a < b { (b, a) } else { (a, b) };

    // 生成 SVG
    let svg = render_svg(left, right, op);

    // 签名
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + CAPTCHA_EXPIRY;
    let token = sign_payload(&Payload { answer, exp });

    Captcha { svg, token }
}

fn render_svg(left: u8, right: u8, op: char) -> String {
    let w: f64 = 200.0;
    let h: f64 = 55.0;
    let cy: f64 = 33.0;
    let base_size = 20.0;

    // 随机字符间距 (不均匀，防 OCR 切割)
    let cx1 = 35.0 + gen_range_f64(-6.0, 6.0);
    let cx2 = 82.0 + gen_range_f64(-6.0, 6.0);
    let cx3 = 128.0 + gen_range_f64(-6.0, 6.0);

    // 每个字符不同字号
    let s1 = base_size + gen_range_f64(-3.0, 3.0);
    let s2 = base_size + gen_range_f64(-3.0, 3.0);
    let s3 = base_size + gen_range_f64(-3.0, 3.0);

    // 随机旋转 ±20°
    let rot1 = gen_range_f64(-20.0, 20.0);
    let rot2 = gen_range_f64(-20.0, 20.0);
    let rot3 = gen_range_f64(-20.0, 20.0);

    // 随机垂直偏移
    let dy1 = gen_range_f64(-4.0, 4.0);
    let dy2 = gen_range_f64(-4.0, 4.0);
    let dy3 = gen_range_f64(-4.0, 4.0);

    // 随机颜色
    let hue = gen_range_i32(200, 260);
    let color1 = format!("hsl({}, {}%, {}%)", hue, gen_range_i32(45, 65), gen_range_i32(25, 40));
    let color2 = format!("hsl({}, {}%, {}%)", gen_range_i32(200, 260), gen_range_i32(45, 65), gen_range_i32(25, 40));

    let mut svg = String::new();
    svg.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">"##
    ));

    // 背景
    let bg1 = "fafbfc";
    let bg2 = "dee2e6";
    svg.push_str(&format!(
        "<rect width=\"{w}\" height=\"{h}\" rx=\"4\" fill=\"#{bg1}\" stroke=\"#{bg2}\" stroke-width=\"1\"/>"
    ));

    // 背景网格线（干扰 OCR 分割）
    for i in 1..4 {
        let gx = i as f64 * 50.0;
        svg.push_str(&format!(
            "<line x1=\"{gx}\" y1=\"2\" x2=\"{gx}\" y2=\"{h}\" stroke=\"#e9ecef\" stroke-width=\"0.5\"/>"
        ));
    }

    // 随机交叉线 (5 条，部分穿过数字区域)
    for _ in 0..5 {
        let lx1 = gen_range_f64(0.0, w);
        let ly1 = gen_range_f64(5.0, h - 5.0);
        let lx2 = lx1 + gen_range_f64(-60.0, 60.0);
        let ly2 = ly1 + gen_range_f64(-25.0, 25.0);
        let sw = gen_range_f64(0.3, 0.8);
        let op = gen_range_f64(0.15, 0.35);
        svg.push_str(&format!(
            "<line x1=\"{lx1}\" y1=\"{ly1}\" x2=\"{lx2}\" y2=\"{ly2}\" stroke=\"#adb5bd\" stroke-width=\"{sw}\" opacity=\"{op}\"/>"
        ));
    }

    // 密集噪点 (25 个)
    for _ in 0..25 {
        let dx = gen_range_f64(8.0, w - 8.0);
        let dy = gen_range_f64(6.0, h - 6.0);
        let dr = gen_range_f64(0.3, 1.5);
        svg.push_str(&format!(
            "<circle cx=\"{dx}\" cy=\"{dy}\" r=\"{dr}\" fill=\"#6c757d\" opacity=\"0.25\"/>"
        ));
    }

    // 伪随机背景数字（干扰 OCR 字符识别）
    for _ in 0..6 {
        let fake_digit = gen_range_i32(0, 9);
        let fx = gen_range_f64(10.0, w - 10.0);
        let fy = gen_range_f64(10.0, h - 8.0);
        let fs = gen_range_f64(8.0, 13.0);
        let fr = gen_range_f64(-30.0, 30.0);
        svg.push_str(&format!(
            "<text x=\"{fx}\" y=\"{fy}\" font-size=\"{fs}\" font-family=\"Courier,monospace\" fill=\"#ced4da\" text-anchor=\"middle\" transform=\"rotate({fr} {fx} {fy})\">{fake_digit}</text>"
        ));
    }

    // 左操作数
    svg.push_str(&format!(
        "<text x=\"{cx1}\" y=\"{cy}\" dy=\"{dy1}\" font-size=\"{s1}\" font-family=\"Courier,monospace\" font-weight=\"bold\" fill=\"{color1}\" text-anchor=\"middle\" transform=\"rotate({rot1} {cx1} {cy})\">{left}</text>"
    ));

    // 运算符
    let op_color = "e03131";
    svg.push_str(&format!(
        "<text x=\"{cx2}\" y=\"{cy}\" dy=\"{dy2}\" font-size=\"{s2}\" font-family=\"Courier,monospace\" font-weight=\"bold\" fill=\"#{op_color}\" text-anchor=\"middle\" transform=\"rotate({rot2} {cx2} {cy})\">{op}</text>"
    ));

    // 右操作数
    svg.push_str(&format!(
        "<text x=\"{cx3}\" y=\"{cy}\" dy=\"{dy3}\" font-size=\"{s3}\" font-family=\"Courier,monospace\" font-weight=\"bold\" fill=\"{color2}\" text-anchor=\"middle\" transform=\"rotate({rot3} {cx3} {cy})\">{right}</text>"
    ));

    // 问号
    svg.push_str(&format!(
        "<text x=\"178\" y=\"{cy}\" font-size=\"{base_size}\" font-family=\"Courier,monospace\" fill=\"#868e96\" text-anchor=\"middle\" opacity=\"0.5\">= ?</text>"
    ));

    svg.push_str("</svg>");
    svg
}

// ── 测试 ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify() {
        let captcha = generate_captcha();
        // 提取答案（从 SVG 解析）……这里直接验证 token 机制
        // 生成后立即用正确答案验证
        let ans = extract_answer_from_svg(&captcha.svg);
        let result = verify_token(&captcha.token, &ans.to_string());
        assert!(result.is_some());
    }

    #[test]
    fn test_wrong_answer() {
        let captcha = generate_captcha();
        let ans = extract_answer_from_svg(&captcha.svg);
        let wrong = (ans + 1).to_string();
        let result = verify_token(&captcha.token, &wrong);
        assert!(result.is_none());
    }

    #[test]
    fn test_expired_token() {
        // 模拟过期 token
        let token = "1000000000:fake";  // 1970 年代，肯定过期
        let result = verify_token(token, "42");
        assert!(result.is_none());
    }

    fn extract_answer_from_svg(svg: &str) -> u8 {
        // 从 SVG 文本中提取操作数和运算符（仅粗体主字符，跳过背景噪音）
        let nums: Vec<u8> = svg
            .split("</text>")
            .filter(|chunk| chunk.contains("font-weight=\"bold\"")) // 只取粗体主字符
            .filter_map(|chunk| {
                let start = chunk.rfind('>')?;
                let text = &chunk[start + 1..];
                let text = text.trim();
                if text.len() <= 2 && text.chars().all(|c| c.is_ascii_digit()) {
                    text.parse().ok()
                } else {
                    None
                }
            })
            .collect();

        let has_plus = svg.contains(">+<");
        let has_minus = svg.contains(">-<");

        match (nums.len(), has_plus, has_minus) {
            (2, true, false) => nums[0] + nums[1],
            (2, false, true) => {
                if nums[0] >= nums[1] { nums[0] - nums[1] } else { nums[1] - nums[0] }
            }
            _ => 0,
        }
    }
}
