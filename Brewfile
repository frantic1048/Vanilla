Dir.glob(File.join(__dir__, "brewfiles", "*.rb")).sort.each do |path|
  instance_eval(File.read(path), path)
end
