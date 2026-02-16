class XcomRs < Formula
  desc "Agent-friendly X.com CLI with introspection and machine-readable output"
  homepage "https://github.com/tumf/xcom-rs"
  url "https://crates.io/api/v1/crates/xcom-rs/0.1.14/download"
  sha256 "a056ea060339494d5edc2b14b9540f24f144e54430980fe7415ebb2aec91388e"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    output = shell_output("#{bin}/xcom-rs commands --output json")
    assert_match '"ok":true', output
  end
end
