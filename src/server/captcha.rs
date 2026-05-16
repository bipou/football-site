use aws_lc_rs::hmac::{Key, sign};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::Rng;

use crate::utils::constant;

const CAPTCHA_EXPIRY: u64 = 300; // 5 分钟

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
    let mut rng = rand::thread_rng();

    // 随机操作数 (1~9)
    let a: u8 = rng.gen_range(1..=9);
    let b: u8 = rng.gen_range(1..=9);
    let op = if rng.gen_bool(0.5) { '+' } else { '-' };
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
    let mut rng = rand::thread_rng();

    // 每个字符的位置
    let cx1 = 50.0;
    let cx2 = 100.0;
    let cx3 = 150.0;

    let cy = 40.0;
    let char_size = 24.0;

    // 每个字符随机微旋转 (-15° ~ +15°)
    let rot1: f64 = rng.gen_range(-15.0..=15.0);
    let rot2: f64 = rng.gen_range(-15.0..=15.0);
    let rot3: f64 = rng.gen_range(-15.0..=15.0);

    // 随机颜色偏移 (深色)
    let hue = rng.gen_range(200..=260);
    let color = format!("hsl({}, 60%, 30%)", hue);

    let mut svg = String::new();
    svg.push_str(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 70" width="200" height="70">"##);

    // 背景
    let hex1 = "f8f9fa";
    let hex2 = "dee2e6";
    svg.push_str(&format!(
        "<rect width=\"200\" height=\"70\" rx=\"4\" fill=\"#{hex1}\" stroke=\"#{hex2}\" stroke-width=\"1\"/>"
    ));

    // 随机噪点线 (3 条)
    for _ in 0..3 {
        let x1 = rng.gen_range(0.0..200.0);
        let y1 = rng.gen_range(10.0..60.0);
        let x2 = x1 + rng.gen_range(-40.0..40.0);
        let y2 = y1 + rng.gen_range(-20.0..20.0);
        let sw = rng.gen_range(0.3..1.0);
        let lc = "adb5bd";
        svg.push_str(&format!(
            "<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"#{lc}\" stroke-width=\"{sw}\" opacity=\"0.4\"/>"
        ));
    }

    // 随机噪点 (15 个)
    for _ in 0..15 {
        let dx = rng.gen_range(10.0..190.0);
        let dy = rng.gen_range(8.0..62.0);
        let dr = rng.gen_range(0.5..1.5);
        let fc = "6c757d";
        svg.push_str(&format!(
            "<circle cx=\"{dx}\" cy=\"{dy}\" r=\"{dr}\" fill=\"#{fc}\" opacity=\"0.3\"/>"
        ));
    }

    // 左操作数
    svg.push_str(&format!(
        "<text x=\"{cx1}\" y=\"{cy}\" font-size=\"{char_size}\" font-family=\"Courier,monospace\" font-weight=\"bold\" fill=\"{color}\" text-anchor=\"middle\" transform=\"rotate({rot1} {cx1} {cy})\">{left}</text>"
    ));

    // 运算符
    let op_color = "e03131";
    svg.push_str(&format!(
        "<text x=\"{cx2}\" y=\"{cy}\" font-size=\"{char_size}\" font-family=\"Courier,monospace\" font-weight=\"bold\" fill=\"#{op_color}\" text-anchor=\"middle\" transform=\"rotate({rot2} {cx2} {cy})\">{op}</text>"
    ));

    // 右操作数
    svg.push_str(&format!(
        "<text x=\"{cx3}\" y=\"{cy}\" font-size=\"{char_size}\" font-family=\"Courier,monospace\" font-weight=\"bold\" fill=\"{color}\" text-anchor=\"middle\" transform=\"rotate({rot3} {cx3} {cy})\">{right}</text>"
    ));

    // 问号（结果位）
    let qc = "868e96";
    svg.push_str(&format!(
        "<text x=\"200\" y=\"{cy}\" font-size=\"{char_size}\" font-family=\"Courier,monospace\" fill=\"#{qc}\" text-anchor=\"middle\" opacity=\"0.5\">= ?</text>"
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
        // 从 SVG 文本中提取操作数和运算符
        // 解析 <text> 元素中的数字
        let nums: Vec<u8> = svg
            .split("</text>")
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
