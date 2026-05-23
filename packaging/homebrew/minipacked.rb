class Minipacked < Formula
  desc "Simple tool to pack files and directories into portable (or even encrypted) containers written in rust"
  homepage "https://github.com/FlyInMyEye/Minipacked"
  url "https://github.com/FlyInMyEye/Minipacked/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "d1ef219f9705dcf3fe12968eaf03e9e2540ff6a9cf26bf3ce2f436617e8f2f2f"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    (testpath/"sample.txt").write("hello\n")
    output = pipe_output("#{bin}/minipack sample.txt", "sample\n\n")
    assert_match "Done:", output
    assert_predicate testpath/"sample.minipacked", :exist?

    rm testpath/"sample.txt"
    unpacked = shell_output("#{bin}/miniunpack sample.minipacked")
    assert_match "Done:", unpacked
    assert_equal "hello\n", (testpath/"sample.txt").read
  end
end
