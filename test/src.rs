<pre><a name="l1"><b>use</b> std::io::prelude::*;
<a name="l2">
<a name="l3"><b>fn</b> to_string(a: &str) String {
<a name="l4">    String::new("abc")
<a name="l5">}
<a name="l6">
<a name="l7"><b>fn</b> foo() -> String {
<a name="l8">    <b>return</b> "abc".<a href="src.rs.html#l3">to_string</a>();
<a name="l9">}
<a name="l10">
<a name="l11"><b>fn</b> main() {
<a name="l12">    println("{}", <a href="src.rs.html#l7">foo</a>());
<a name="l13">}
<a name="l14"></pre>