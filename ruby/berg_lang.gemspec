$:.unshift(File.dirname(__FILE__) + '/lib')
require 'berg_lang/version'

Gem::Specification.new do |s|
  s.name = 'berg_lang'
  s.version = BergLang::VERSION
  s.platform = Gem::Platform::RUBY
  s.licenses = ['MIT']
  s.extra_rdoc_files = ['../LICENSE']
  s.summary = 'A pervasively lazy, zero ceremony, statically typed language.'
  s.description = s.summary
  s.author = 'John Keiser'
  s.email = 'john@johnkeiser.com'
#  s.homepage = 'http://berg-lang.org'

  s.required_ruby_version = ">= 2.3"

  s.add_development_dependency 'rake'
  s.add_development_dependency 'rspec'

  s.bindir       = "bin"
  s.executables  = %w( berg )

  s.require_path = 'lib'
  s.files = %w(Gemfile ../LICENSE .rspec) + Dir.glob("*.gemspec") +
      Dir.glob("{distro,lib,tasks,spec}/**/*", File::FNM_DOTMATCH).reject {|f| File.directory?(f) }
end
