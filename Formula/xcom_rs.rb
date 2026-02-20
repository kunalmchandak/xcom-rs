class XcomRs < Formula
  desc "Agent-friendly X.com CLI with introspection and machine-readable output"
  homepage "https://github.com/tumf/xcom-rs"
  url "https://crates.io/api/v1/crates/xcom-rs/0.1.25/download"
  sha256 "7ab00d0075268ca29e8f67228aa183e1603beb9cdc40ef987233d8c4dc858f49"
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
