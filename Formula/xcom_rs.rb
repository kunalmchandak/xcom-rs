class XcomRs < Formula
  desc "Agent-friendly X.com CLI with introspection and machine-readable output"
  homepage "https://github.com/tumf/xcom-rs"
  url "https://crates.io/api/v1/crates/xcom-rs/0.1.24/download"
  sha256 "8946955e09774d3c10cc9c93888488c994f565cd0e359e8801b456b673a41315"
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
