// Analysis engine for comprehensive metrics computation
use crate::core::evaluation::*;
use crate::utils::error::Result;
use std::collections::HashMap;
use regex::Regex;

pub struct AnalysisEngine {
    sentiment_analyzer: SentimentAnalyzer,
    similarity_calculator: SimilarityCalculator,
    content_analyzer: ContentAnalyzer,
    quality_assessor: QualityAssessor,
}

impl AnalysisEngine {
    pub fn new() -> Self {
        Self {
            sentiment_analyzer: SentimentAnalyzer::new(),
            similarity_calculator: SimilarityCalculator::new(),
            content_analyzer: ContentAnalyzer::new(),
            quality_assessor: QualityAssessor::new(),
        }
    }
    
    pub async fn analyze_results(
        &self,
        results: &[ExecutionResult],
        options: &AnalysisOptions,
    ) -> Result<AnalysisResults> {
        let mut analysis = AnalysisResults {
            response_metrics: Vec::new(),
            similarity_matrix: Vec::new(),
            content_analysis: Vec::new(),
            quality_indicators: Vec::new(),
            cost_breakdown: self.compute_cost_breakdown(results)?,
            performance_stats: self.compute_performance_stats(results)?,
        };
        
        // Compute response metrics
        if options.response_metrics {
            analysis.response_metrics = self.compute_response_metrics(results).await?;
        }
        
        // Compute similarity analysis
        if options.similarity_analysis {
            analysis.similarity_matrix = self.compute_similarity_matrix(results).await?;
        }
        
        // Compute content analysis
        if options.content_analysis {
            analysis.content_analysis = self.compute_content_analysis(results).await?;
        }
        
        // Compute quality indicators
        if options.quality_indicators {
            analysis.quality_indicators = self.compute_quality_indicators(results).await?;
        }
        
        Ok(analysis)
    }
    
    async fn compute_response_metrics(&self, results: &[ExecutionResult]) -> Result<Vec<ResponseMetrics>> {
        let mut metrics = Vec::new();
        
        for result in results {
            if matches!(result.status, ExecutionStatus::Success) {
                let content = &result.output;
                
                // Basic text metrics
                let length_chars = content.len();
                let length_words = content.split_whitespace().count();
                let length_sentences = self.count_sentences(content);
                
                // Readability score (Flesch reading ease approximation)
                let readability_score = self.calculate_readability_score(content);
                
                // Sentiment analysis
                let sentiment_score = self.sentiment_analyzer.analyze(content);
                
                // Confidence score based on response characteristics
                let confidence_score = self.calculate_confidence_score(content);
                
                metrics.push(ResponseMetrics {
                    execution_id: result.id.clone(),
                    length_chars,
                    length_words,
                    length_sentences,
                    readability_score,
                    sentiment_score,
                    confidence_score,
                    response_time_ms: result.metadata.response_time_ms,
                    cost_usd: result.metadata.cost_usd,
                });
            }
        }
        
        Ok(metrics)
    }
    
    async fn compute_similarity_matrix(&self, results: &[ExecutionResult]) -> Result<Vec<SimilarityScore>> {
        let mut similarities = Vec::new();
        let successful_results: Vec<_> = results.iter()
            .filter(|r| matches!(r.status, ExecutionStatus::Success))
            .collect();
        
        for (i, result1) in successful_results.iter().enumerate() {
            for (j, result2) in successful_results.iter().enumerate() {
                if i < j {  // Only compute upper triangle to avoid duplicates
                    let similarity = self.similarity_calculator.calculate_similarity(
                        &result1.output,
                        &result2.output,
                        SimilarityType::Cosine,
                    );
                    
                    similarities.push(SimilarityScore {
                        execution_id_1: result1.id.clone(),
                        execution_id_2: result2.id.clone(),
                        similarity_score: similarity,
                        similarity_type: SimilarityType::Cosine,
                    });
                }
            }
        }
        
        Ok(similarities)
    }
    
    async fn compute_content_analysis(&self, results: &[ExecutionResult]) -> Result<Vec<ContentAnalysis>> {
        let mut analyses = Vec::new();
        
        for result in results {
            if matches!(result.status, ExecutionStatus::Success) {
                let content = &result.output;
                
                // Extract keywords
                let keywords = self.content_analyzer.extract_keywords(content);
                
                // Extract entities (simple implementation)
                let entities = self.content_analyzer.extract_entities(content);
                
                // Extract topics
                let topics = self.content_analyzer.extract_topics(content);
                
                // Language quality assessment
                let language_quality = self.content_analyzer.assess_language_quality(content);
                
                analyses.push(ContentAnalysis {
                    execution_id: result.id.clone(),
                    keywords,
                    entities,
                    topics,
                    language_quality,
                });
            }
        }
        
        Ok(analyses)
    }
    
    async fn compute_quality_indicators(&self, results: &[ExecutionResult]) -> Result<Vec<QualityScore>> {
        let mut quality_scores = Vec::new();
        
        for result in results {
            if matches!(result.status, ExecutionStatus::Success) {
                let content = &result.output;
                
                // Assess quality dimensions
                let relevance_score = self.quality_assessor.assess_relevance(content, &result.input);
                let accuracy_score = self.quality_assessor.assess_accuracy(content);
                let helpfulness_score = self.quality_assessor.assess_helpfulness(content);
                
                let overall_score = (relevance_score + accuracy_score + helpfulness_score) / 3.0;
                
                quality_scores.push(QualityScore {
                    execution_id: result.id.clone(),
                    relevance_score,
                    accuracy_score,
                    helpfulness_score,
                    overall_score,
                });
            }
        }
        
        Ok(quality_scores)
    }
    
    fn compute_cost_breakdown(&self, results: &[ExecutionResult]) -> Result<CostBreakdown> {
        let mut cost_by_provider = HashMap::new();
        let mut cost_by_model = HashMap::new();
        let mut input_token_cost = 0.0;
        let mut output_token_cost = 0.0;
        
        for result in results {
            // Cost by provider
            *cost_by_provider.entry(result.provider.clone()).or_insert(0.0) += result.metadata.cost_usd;
            
            // Cost by model
            let model_key = format!("{}/{}", result.provider, result.model);
            *cost_by_model.entry(model_key).or_insert(0.0) += result.metadata.cost_usd;
            
            // Estimate input/output token costs (rough approximation)
            let total_tokens = result.metadata.token_count_input + result.metadata.token_count_output;
            if total_tokens > 0 {
                let input_ratio = result.metadata.token_count_input as f64 / total_tokens as f64;
                input_token_cost += result.metadata.cost_usd * input_ratio;
                output_token_cost += result.metadata.cost_usd * (1.0 - input_ratio);
            }
        }
        
        let total_cost = results.iter().map(|r| r.metadata.cost_usd).sum();
        
        Ok(CostBreakdown {
            total_cost,
            cost_by_provider,
            cost_by_model,
            input_token_cost,
            output_token_cost,
        })
    }
    
    fn compute_performance_stats(&self, results: &[ExecutionResult]) -> Result<PerformanceStats> {
        let response_times: Vec<u64> = results.iter()
            .map(|r| r.metadata.response_time_ms)
            .collect();
        
        let successful_executions = results.iter()
            .filter(|r| matches!(r.status, ExecutionStatus::Success))
            .count() as u32;
        
        let total_executions = results.len() as u32;
        let failed_executions = total_executions - successful_executions;
        let success_rate = if total_executions > 0 {
            (successful_executions as f32 / total_executions as f32) * 100.0
        } else {
            0.0
        };
        
        let avg_response_time = if !response_times.is_empty() {
            response_times.iter().sum::<u64>() as f64 / response_times.len() as f64
        } else {
            0.0
        };
        
        let median_response_time = {
            let mut sorted_times = response_times.clone();
            sorted_times.sort();
            if sorted_times.is_empty() {
                0.0
            } else if sorted_times.len() % 2 == 0 {
                let mid = sorted_times.len() / 2;
                (sorted_times[mid - 1] + sorted_times[mid]) as f64 / 2.0
            } else {
                sorted_times[sorted_times.len() / 2] as f64
            }
        };
        
        let min_response_time = response_times.iter().min().copied().unwrap_or(0);
        let max_response_time = response_times.iter().max().copied().unwrap_or(0);
        
        Ok(PerformanceStats {
            avg_response_time,
            median_response_time,
            min_response_time,
            max_response_time,
            success_rate,
            total_executions,
            failed_executions,
        })
    }
    
    fn count_sentences(&self, text: &str) -> usize {
        let sentence_endings = Regex::new(r"[.!?]+").unwrap();
        sentence_endings.find_iter(text).count()
    }
    
    fn calculate_readability_score(&self, text: &str) -> f32 {
        let words = text.split_whitespace().count();
        let sentences = self.count_sentences(text).max(1);
        let syllables = self.estimate_syllables(text);
        
        if words == 0 || sentences == 0 {
            return 0.0;
        }
        
        // Flesch Reading Ease formula approximation
        let avg_sentence_length = words as f32 / sentences as f32;
        let avg_syllables_per_word = syllables as f32 / words as f32;
        
        206.835 - (1.015 * avg_sentence_length) - (84.6 * avg_syllables_per_word)
    }
    
    fn estimate_syllables(&self, text: &str) -> usize {
        let vowel_groups = Regex::new(r"[aeiouyAEIOUY]+").unwrap();
        text.split_whitespace()
            .map(|word| vowel_groups.find_iter(word).count().max(1))
            .sum()
    }
    
    fn calculate_confidence_score(&self, text: &str) -> f32 {
        let mut score = 50.0; // Base score
        
        // Increase score for longer, more detailed responses
        if text.len() > 100 {
            score += 10.0;
        }
        if text.len() > 500 {
            score += 10.0;
        }
        
        // Increase score for structured content
        if text.contains('\n') || text.contains('-') || text.contains('•') {
            score += 10.0;
        }
        
        // Decrease score for uncertainty markers
        let uncertainty_markers = ["maybe", "perhaps", "might", "could be", "I think", "possibly"];
        for marker in uncertainty_markers {
            if text.to_lowercase().contains(marker) {
                score -= 5.0;
            }
        }
        
        // Increase score for definitive language
        let definitive_markers = ["definitely", "certainly", "clearly", "exactly", "precisely"];
        for marker in definitive_markers {
            if text.to_lowercase().contains(marker) {
                score += 5.0;
            }
        }
        
        score.max(0.0).min(100.0)
    }
}

// Helper structs for analysis components
pub struct SentimentAnalyzer;

impl SentimentAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn analyze(&self, text: &str) -> f32 {
        // Simple sentiment analysis based on positive/negative word counts
        let positive_words = ["good", "great", "excellent", "amazing", "wonderful", "fantastic", 
                              "love", "like", "enjoy", "happy", "pleased", "satisfied"];
        let negative_words = ["bad", "terrible", "awful", "horrible", "hate", "dislike", 
                              "sad", "angry", "frustrated", "disappointed", "poor", "worst"];
        
        let text_lower = text.to_lowercase();
        let positive_count = positive_words.iter()
            .map(|word| text_lower.matches(word).count())
            .sum::<usize>() as f32;
        let negative_count = negative_words.iter()
            .map(|word| text_lower.matches(word).count())
            .sum::<usize>() as f32;
        
        if positive_count + negative_count == 0.0 {
            0.0 // Neutral
        } else {
            (positive_count - negative_count) / (positive_count + negative_count)
        }
    }
}

pub struct SimilarityCalculator;

impl SimilarityCalculator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn calculate_similarity(&self, text1: &str, text2: &str, similarity_type: SimilarityType) -> f32 {
        match similarity_type {
            SimilarityType::Jaccard => self.jaccard_similarity(text1, text2),
            SimilarityType::Cosine => self.cosine_similarity(text1, text2),
            _ => self.simple_similarity(text1, text2),
        }
    }
    
    fn jaccard_similarity(&self, text1: &str, text2: &str) -> f32 {
        let words1: std::collections::HashSet<_> = text1.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        let words2: std::collections::HashSet<_> = text2.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
    
    fn cosine_similarity(&self, text1: &str, text2: &str) -> f32 {
        use similar::{TextDiff, ChangeTag};
        
        let diff = TextDiff::from_lines(text1, text2);
        let total_lines = text1.lines().count().max(text2.lines().count());
        
        if total_lines == 0 {
            return 1.0;
        }
        
        let changes = diff.iter_all_changes().count();
        let equal_changes = diff.iter_all_changes()
            .filter(|change| change.tag() == ChangeTag::Equal)
            .count();
        
        if changes == 0 {
            1.0
        } else {
            equal_changes as f32 / changes as f32
        }
    }
    
    fn simple_similarity(&self, text1: &str, text2: &str) -> f32 {
        // Simple character-based similarity
        let len1 = text1.len();
        let len2 = text2.len();
        let max_len = len1.max(len2);
        
        if max_len == 0 {
            return 1.0;
        }
        
        let common_chars = text1.chars()
            .zip(text2.chars())
            .filter(|(c1, c2)| c1 == c2)
            .count();
        
        common_chars as f32 / max_len as f32
    }
}

pub struct ContentAnalyzer;

impl ContentAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn extract_keywords(&self, text: &str) -> Vec<String> {
        // Simple keyword extraction based on word frequency
        let mut word_counts = HashMap::new();
        let stop_words = ["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", 
                          "of", "with", "by", "from", "up", "about", "into", "through", "during"];
        
        for word in text.split_whitespace() {
            let word = word.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            
            if word.len() > 3 && !stop_words.contains(&word.as_str()) {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }
        
        let mut keywords: Vec<_> = word_counts.into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(word, _)| word)
            .collect();
        keywords.sort();
        keywords.truncate(10); // Top 10 keywords
        keywords
    }
    
    pub fn extract_entities(&self, text: &str) -> Vec<Entity> {
        // Simple entity extraction (capitalized words that aren't at sentence start)
        let mut entities = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
            
            if clean_word.len() > 1 
                && clean_word.chars().next().unwrap().is_uppercase()
                && (i == 0 || !words[i-1].ends_with('.')) // Not sentence start
            {
                entities.push(Entity {
                    text: clean_word.to_string(),
                    entity_type: "UNKNOWN".to_string(),
                    confidence: 0.7,
                });
            }
        }
        
        entities.truncate(5); // Top 5 entities
        entities
    }
    
    pub fn extract_topics(&self, text: &str) -> Vec<Topic> {
        // Simple topic extraction based on domain keywords
        let mut topics = Vec::new();
        
        let domain_keywords = vec![
            ("Technology", vec!["software", "computer", "digital", "tech", "programming", "code"]),
            ("Business", vec!["company", "market", "business", "sales", "revenue", "profit"]),
            ("Science", vec!["research", "study", "analysis", "data", "experiment", "theory"]),
            ("Health", vec!["health", "medical", "doctor", "treatment", "patient", "disease"]),
        ];
        
        let text_lower = text.to_lowercase();
        
        for (topic_name, keywords) in domain_keywords {
            let matches = keywords.iter()
                .map(|keyword| text_lower.matches(keyword).count())
                .sum::<usize>();
            
            if matches > 0 {
                let confidence = (matches as f32 / text.split_whitespace().count() as f32).min(1.0);
                topics.push(Topic {
                    name: topic_name.to_string(),
                    confidence,
                    keywords: keywords.iter().map(|s| s.to_string()).collect(),
                });
            }
        }
        
        topics.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        topics.truncate(3); // Top 3 topics
        topics
    }
    
    pub fn assess_language_quality(&self, text: &str) -> LanguageQuality {
        // Simple language quality assessment
        let word_count = text.split_whitespace().count();
        let avg_word_length = if word_count > 0 {
            text.split_whitespace()
                .map(|w| w.len())
                .sum::<usize>() as f32 / word_count as f32
        } else {
            0.0
        };
        
        // Grammar score based on basic indicators
        let grammar_score = if text.matches('.').count() > 0 
            && text.chars().next().map_or(false, |c| c.is_uppercase()) {
            80.0
        } else {
            60.0
        };
        
        // Clarity score based on sentence length and structure
        let sentences = text.split('.').count();
        let avg_sentence_length = if sentences > 0 {
            word_count as f32 / sentences as f32
        } else {
            0.0
        };
        
        let clarity_score = if avg_sentence_length > 10.0 && avg_sentence_length < 25.0 {
            85.0
        } else {
            70.0
        };
        
        // Coherence score based on text structure
        let coherence_score = if text.contains('\n') || text.len() > 200 {
            80.0
        } else {
            75.0
        };
        
        // Completeness score based on length and content
        let completeness_score = if word_count > 50 && text.contains('.') {
            85.0
        } else {
            70.0
        };
        
        LanguageQuality {
            grammar_score,
            clarity_score,
            coherence_score,
            completeness_score,
        }
    }
}

pub struct QualityAssessor;

impl QualityAssessor {
    pub fn new() -> Self {
        Self
    }
    
    pub fn assess_relevance(&self, response: &str, prompt: &str) -> f32 {
        // Simple relevance assessment based on keyword overlap
        let response_words: std::collections::HashSet<_> = response
            .split_whitespace()
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| w.len() > 3)
            .collect();
        
        let prompt_words: std::collections::HashSet<_> = prompt
            .split_whitespace()
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| w.len() > 3)
            .collect();
        
        if prompt_words.is_empty() {
            return 50.0;
        }
        
        let overlap = response_words.intersection(&prompt_words).count();
        let relevance = (overlap as f32 / prompt_words.len() as f32) * 100.0;
        
        relevance.min(100.0)
    }
    
    pub fn assess_accuracy(&self, response: &str) -> f32 {
        // Simple accuracy assessment based on response characteristics
        let mut score = 70.0; // Base score
        
        // Increase score for specific, detailed responses
        if response.len() > 100 {
            score += 10.0;
        }
        
        // Increase score for structured responses
        if response.contains('\n') || response.contains('-') {
            score += 10.0;
        }
        
        // Decrease score for uncertainty markers
        let uncertainty_patterns = ["not sure", "maybe", "might be", "probably"];
        for pattern in uncertainty_patterns {
            if response.to_lowercase().contains(pattern) {
                score -= 5.0;
            }
        }
        
        score.max(0.0).min(100.0)
    }
    
    pub fn assess_helpfulness(&self, response: &str) -> f32 {
        // Simple helpfulness assessment
        let mut score = 60.0; // Base score
        
        // Increase score for actionable content
        let action_words = ["how to", "steps", "follow", "do this", "try", "use"];
        for word in action_words {
            if response.to_lowercase().contains(word) {
                score += 5.0;
            }
        }
        
        // Increase score for examples
        if response.to_lowercase().contains("example") || response.contains("e.g.") {
            score += 10.0;
        }
        
        // Increase score for comprehensive responses
        if response.len() > 200 {
            score += 10.0;
        }
        
        score.max(0.0).min(100.0)
    }
}