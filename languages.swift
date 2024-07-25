enum Language {    
    case c, cpp, go, swift, rust, python, ruby, lua, shell, unknown
}

enum CommentPattern: String {
    case cLike = "//"
    case pyLike = "#"
    case lua = "--"
    case shell = ";"
    case unknown 
}

func detectLanguage(filename: String) -> Language {
    var result: Language
    if filename.contains("cpp") || filename.contains("cc") { result = .cpp }
    else if filename.contains("c") { result = .c }
    else if filename.contains("go") { result = .go }
    else if filename.contains("swift") { result = .swift }
    else if filename.contains("rs") { result = .rust }
    else if filename.contains("py") { result = .python }
    else if filename.contains("rb") { result = .ruby }
    else if filename.contains("lua") { result = .lua }
    else if filename.contains("sh") { result = .shell }
    else { result = .unknown }
    return result
}

func determineCommentStyle(lang: Language) -> CommentPattern {
    var result: CommentPattern
    switch lang {
    case .c, .cpp, .go, .swift, .rust:
        result = .cLike
    case .python, .ruby:
        result = .pyLike
    case .lua:
        result = .lua
    case .shell:
        result = .shell
    default:
        result = .unknown
    }
    return result
}
