use failure::{Error,bail};
use pest::iterators::Pair;

use crate::document_tree::{
    Element,HasChildren,ExtraAttributes,
    elements as e,
    element_categories as c,
    attribute_types::{ID,NameToken},
    extra_attributes as a,
};

use crate::parser::{
    pest_rst::Rule,
    pair_ext_parse::PairExt,
};
use super::inline::convert_inline;


pub fn convert_ssubel(pair: Pair<Rule>) -> Result<Option<c::StructuralSubElement>, Error> {
    // TODO: This is just a proof of concept. Keep closely to DTD in final version!
    Ok(Some(match pair.as_rule() {
        Rule::title            => convert_title(pair).into(),
        Rule::paragraph        => convert_paragraph(pair)?.into(),
        Rule::target           => convert_target(pair)?.into(),
        Rule::substitution_def => convert_substitution_def(pair)?.into(),
        Rule::admonition_gen   => convert_admonition_gen(pair)?.into(),
        Rule::image            => convert_image::<e::Image>(pair)?.into(),
        Rule::EOI              => return Ok(None),
        rule => panic!("unknown rule {:?}", rule),
    }))
}


fn convert_title(pair: Pair<Rule>) -> e::Title {
    let mut title: Option<&str> = None;
    let mut _adornment_char: Option<char> = None;
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::line => title = Some(p.as_str()),
            Rule::adornments => _adornment_char = Some(p.as_str().chars().next().expect("Empty adornment?")),
            rule => unimplemented!("Unexpected rule in title: {:?}", rule),
        };
    }
    // TODO adornment char
    e::Title::with_children(vec![
        title.expect("No text in title").into()
    ])
}


fn convert_paragraph(pair: Pair<Rule>) -> Result<e::Paragraph, Error> {
    let children = pair.into_inner().map(convert_inline).collect::<Result<_,_>>()?;
    Ok(e::Paragraph::with_children(children))
}


fn convert_target(pair: Pair<Rule>) -> Result<e::Target, Error> {
    let mut attrs = a::Target {
        anonymous: false,
        ..Default::default()
    };
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::target_name_uq | Rule::target_name_qu => {
                //TODO: abstract away
                attrs.refid = Some(       ID(p.as_str().to_owned().replace(' ', "-")));
                attrs.refname.push(NameToken(p.as_str().to_owned()));
            },
            Rule::link_target => attrs.refuri = Some(p.parse()?),
            rule => panic!("Unexpected rule in target: {:?}", rule),
        }
    }
    Ok(e::Target::new(Default::default(), attrs))
}

fn convert_substitution_def(pair: Pair<Rule>) -> Result<e::SubstitutionDefinition, Error> {
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str();  // Rule::substitution_name
    let inner_pair = pairs.next().unwrap();
    let inner: c::TextOrInlineElement = match inner_pair.as_rule() {
        Rule::image => convert_image::<e::ImageInline>(inner_pair)?.into(),
        rule => panic!("Unknown substitution rule {:?}", rule),
    };
    let mut subst_def = e::SubstitutionDefinition::with_children(vec![inner]);
    subst_def.names_mut().push(NameToken(name.to_owned()));
    Ok(subst_def)
}

fn convert_image<I>(pair: Pair<Rule>) -> Result<I, Error> where I: Element + ExtraAttributes<a::Image> {
    let mut pairs = pair.into_inner();
    let mut image = I::with_extra(a::Image::new(
        pairs.next().unwrap().parse()?,  // line
    ));
    if let Some(opt_block) = pairs.next() {  // image_opt_block
        let options = opt_block.into_inner();
        for opt in options {
            let mut opt_iter = opt.into_inner();
            let opt_name = opt_iter.next().unwrap();
            let opt_val = opt_iter.next().unwrap();
            match opt_name.as_str() {
                "class"  => image.classes_mut().push(opt_val.as_str().to_owned()),
                "name"   => image.names_mut().push(NameToken(opt_val.as_str().to_owned())),
                "alt"    => image.extra_mut().alt    = Some(opt_val.as_str().to_owned()),
                "height" => image.extra_mut().height = Some(opt_val.parse()?),
                "width"  => image.extra_mut().width  = Some(opt_val.parse()?),
                "scale"  => image.extra_mut().scale  = Some(parse_scale(&opt_val)?),
                "align"  => image.extra_mut().align  = Some(opt_val.parse()?),
                "target" => image.extra_mut().target = Some(opt_val.parse()?),
                name => bail!("Unknown Image option {}", name),
            }
        }
    }
    Ok(image)
}

fn parse_scale(pair: &Pair<Rule>) -> Result<u8, Error> {
    let input = if pair.as_str().chars().rev().next() == Some('%') { &pair.as_str()[..pair.as_str().len()-1] } else { pair.as_str() };
    use pest::error::{Error,ErrorVariant};
    Ok(input.parse().map_err(|e: std::num::ParseIntError| {
        let var: ErrorVariant<Rule> = ErrorVariant::CustomError { message: e.to_string() };
        Error::new_from_span(var, pair.as_span())
    })?)
}

fn convert_admonition_gen(pair: Pair<Rule>) -> Result<c::BodyElement, Error> {
    let mut iter = pair.into_inner();
    let typ = iter.next().unwrap().as_str();
    // TODO: in reality it contains body elements.
    let children: Vec<c::BodyElement> = iter.map(|p| e::Paragraph::with_children(vec![p.as_str().into()]).into()).collect();
    Ok(match typ {
        "attention" => e::Attention::with_children(children).into(),
        "hint"      =>      e::Hint::with_children(children).into(),
        "note"      =>      e::Note::with_children(children).into(),
        "caution"   =>   e::Caution::with_children(children).into(),
        "danger"    =>    e::Danger::with_children(children).into(),
        "error"     =>     e::Error::with_children(children).into(),
        "important" => e::Important::with_children(children).into(),
        "tip"       =>       e::Tip::with_children(children).into(),
        "warning"   =>   e::Warning::with_children(children).into(),
        typ         => panic!("Unknown admontion type {}!", typ),
    })
}
