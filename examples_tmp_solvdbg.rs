fn main() {
    let env = epher_core::Env::default();
    let out = epher_core::run(&epher_core::parse_script("solve sin(x) == 0").unwrap(), &mut env.clone());
    println!("{:?}", out);
    let out2 = epher_core::run(&epher_core::parse_script("a = 2; solve a*x == 6").unwrap(), &mut env.clone());
    println!("{:?}", out2);
    let mut env2 = epher_core::Env::default();
    let out3 = epher_core::run(&epher_core::parse_script("solve a*x == 6").unwrap(), &mut env2);
    println!("{:?}", out3);
}
