#!/usr/bin/env python3
"""
Extract g2pM model weights and convert to Rust-compatible format
"""
import pickle
import json
import numpy as np
import os

def extract_g2pm_weights():
    base_path = os.path.dirname(os.path.abspath(__file__))
    
    # Load pickle files
    with open(os.path.join(base_path, 'char2idx.pkl'), 'rb') as f:
        char2idx = pickle.load(f)
    
    with open(os.path.join(base_path, 'class2idx.pkl'), 'rb') as f:
        class2idx = pickle.load(f)
    
    with open(os.path.join(base_path, 'digest_cedict.pkl'), 'rb') as f:
        cedict = pickle.load(f)
    
    with open(os.path.join(base_path, 'np_ckpt.pkl'), 'rb') as f:
        state_dict = pickle.load(f)
    
    # Extract model dimensions
    embeddings = state_dict["embedding.weight"]
    vocab_size, embed_dim = embeddings.shape
    hidden_size = state_dict["lstm.weight_hh_l0"].shape[1]
    num_classes = state_dict["logit_layer.1.weight"].shape[0]
    
    print(f"Model dimensions:")
    print(f"  Vocab size: {vocab_size}")
    print(f"  Embed dim: {embed_dim}")
    print(f"  Hidden size: {hidden_size}")
    print(f"  Num classes: {num_classes}")
    
    # Save as JSON for Rust to load
    output = {
        "vocab_size": vocab_size,
        "embed_dim": embed_dim,
        "hidden_size": hidden_size,
        "num_classes": num_classes,
        "char2idx": char2idx,
        "idx2class": {str(v): k for k, v in class2idx.items()},
        "cedict": {k: v for k, v in cedict.items()},
    }
    
    with open('g2pm_model_info.json', 'w', encoding='utf-8') as f:
        json.dump(output, f, ensure_ascii=False, indent=2)
    
    # Save weights as numpy arrays (will be converted to Rust constants)
    np.save('embeddings.npy', embeddings)
    np.save('lstm_weight_ih.npy', state_dict["lstm.weight_ih_l0"])
    np.save('lstm_weight_hh.npy', state_dict["lstm.weight_hh_l0"])
    np.save('lstm_bias_ih.npy', state_dict["lstm.bias_ih_l0"])
    np.save('lstm_bias_hh.npy', state_dict["lstm.bias_hh_l0"])
    np.save('lstm_weight_ih_reverse.npy', state_dict["lstm.weight_ih_l0_reverse"])
    np.save('lstm_weight_hh_reverse.npy', state_dict["lstm.weight_hh_l0_reverse"])
    np.save('lstm_bias_ih_reverse.npy', state_dict["lstm.bias_ih_l0_reverse"])
    np.save('lstm_bias_hh_reverse.npy', state_dict["lstm.bias_hh_l0_reverse"])
    np.save('fc_weight_l0.npy', state_dict["logit_layer.0.weight"])
    np.save('fc_bias_l0.npy', state_dict["logit_layer.0.bias"])
    np.save('fc_weight_l1.npy', state_dict["logit_layer.1.weight"])
    np.save('fc_bias_l1.npy', state_dict["logit_layer.1.bias"])
    
    print("\nWeights extracted successfully!")
    print(f"Total parameters: {sum(p.size for p in state_dict.values())}")

if __name__ == '__main__':
    extract_g2pm_weights()
