class StringHelper
  def self.truncate(text, length)
    text[0...length]
  end

  def self.capitalize_words(text)
    text.split.map(&:capitalize).join(' ')
  end
end

def format_output(data)
  data.to_s.strip
end
