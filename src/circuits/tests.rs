#[cfg(test)]
mod test {

    use crate::circuits::merkle_sum_tree::MerkleSumTreeCircuit;
    use crate::circuits::utils::{full_prover, full_verifier};
    use crate::merkle_sum_tree::big_int_to_fp;
    use halo2_proofs::{
        dev::{FailureLocation, MockProver, VerifyFailure},
        halo2curves::bn256::{Bn256, Fr as Fp},
        plonk::{keygen_pk, keygen_vk, Any},
        poly::kzg::commitment::ParamsKZG,
    };
    use num_bigint::ToBigInt;
    use rand::rngs::OsRng;

    #[test]
    fn test_valid_merkle_sum_tree() {
        let assets_sum = Fp::from(556863u64); // greater than liabilities sum (556862)

        let circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        let public_input = vec![
            circuit.leaf_hash,
            circuit.leaf_balance,
            circuit.root_hash,
            circuit.assets_sum,
        ];

        let valid_prover = MockProver::run(9, &circuit, vec![public_input]).unwrap();

        valid_prover.assert_satisfied();
    }

    #[test]
    fn test_valid_merkle_sum_tree_2() {
        // Same as above but now the entries contain a balance that is greater than 64 bits
        // liabilities sum is 18446744073710096590

        let assets_sum_big_int = 18446744073710096591_u128.to_bigint().unwrap(); // greater than liabilities sum

        let assets_sum = big_int_to_fp(&assets_sum_big_int);

        let circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        let public_input = vec![
            circuit.leaf_hash,
            circuit.leaf_balance,
            circuit.root_hash,
            circuit.assets_sum,
        ];

        let valid_prover = MockProver::run(9, &circuit, vec![public_input]).unwrap();

        valid_prover.assert_satisfied();
    }

    #[test]
    fn test_valid_merkle_sum_tree_with_full_prover() {
        let assets_sum = Fp::from(556863u64); // greater than liabilities sum (556862)

        let circuit = MerkleSumTreeCircuit::init_empty();

        // we generate a universal trusted setup of our own for testing
        let params = ParamsKZG::<Bn256>::setup(9, OsRng);

        // we generate the verification key and the proving key
        // we use an empty circuit just to enphasize that the circuit input are not relevant when generating the keys
        // Note: the dimension of the circuit used to generate the keys must be the same as the dimension of the circuit used to generate the proof
        // In this case, the dimension are represented by the heigth of the merkle tree
        let vk = keygen_vk(&params, &circuit).expect("vk generation should not fail");
        let pk = keygen_pk(&params, vk.clone(), &circuit).expect("pk generation should not fail");

        // Only now we can instantiate the circuit with the actual inputs
        let circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        let public_input = vec![
            circuit.leaf_hash,
            circuit.leaf_balance,
            circuit.root_hash,
            circuit.assets_sum,
        ];

        // Generate the proof
        let proof = full_prover(&params, &pk, circuit, &public_input);

        // verify the proof to be true
        assert!(full_verifier(&params, &vk, proof, &public_input));
    }

    // Passing an invalid root hash in the instance column should fail the permutation check between the computed root hash and the instance column root hash
    #[test]
    fn test_invalid_root_hash() {
        let assets_sum = Fp::from(556863u64); // greater than liabilities sum (556862)

        let circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        let invalid_root_hash = Fp::from(1000u64);

        let public_input = vec![
            circuit.leaf_hash,
            circuit.leaf_balance,
            invalid_root_hash,
            circuit.assets_sum,
        ];

        let invalid_prover = MockProver::run(9, &circuit, vec![public_input]).unwrap();

        assert_eq!(
            invalid_prover.verify(),
            Err(vec![
                VerifyFailure::Permutation {
                    column: (Any::Instance, 0).into(),
                    location: FailureLocation::OutsideRegion { row: 2 }
                },
                VerifyFailure::Permutation {
                    column: (Any::advice(), 5).into(),
                    location: FailureLocation::InRegion {
                        region: (16, "permute state").into(),
                        offset: 38
                    }
                }
            ])
        );
    }

    #[test]
    fn test_invalid_root_hash_with_full_prover() {
        let assets_sum = Fp::from(556863u64); // greater than liabilities sum (556862)

        let circuit = MerkleSumTreeCircuit::init_empty();

        // we generate a universal trusted setup of our own for testing
        let params = ParamsKZG::<Bn256>::setup(9, OsRng);

        // we generate the verification key and the proving key
        // we use an empty circuit just to enphasize that the circuit input are not relevant when generating the keys
        let vk = keygen_vk(&params, &circuit).expect("vk should not fail");
        let pk = keygen_pk(&params, vk.clone(), &circuit).expect("pk should not fail");

        // Only now we can instantiate the circuit with the actual inputs
        let circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        let invalid_root_hash = Fp::from(1000u64);

        let public_input = vec![
            circuit.leaf_hash,
            circuit.leaf_balance,
            invalid_root_hash,
            circuit.assets_sum,
        ];

        // Generate the proof
        let proof = full_prover(&params, &pk, circuit, &public_input);

        // verify the proof to be false
        assert!(!full_verifier(&params, &vk, proof, &public_input));
    }

    // Passing an invalid leaf hash as input for the witness generation should fail the permutation check between the computed root hash and the instance column root hash
    #[test]
    fn test_invalid_leaf_hash_as_witness() {
        let assets_sum = Fp::from(556863u64); // greater than liabilities sum (556862)

        let mut circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        // invalidate leaf hash
        circuit.leaf_hash = Fp::from(1000u64);

        let public_input = vec![
            circuit.leaf_hash,
            circuit.leaf_balance,
            circuit.root_hash,
            circuit.assets_sum,
        ];

        let invalid_prover = MockProver::run(9, &circuit, vec![public_input]).unwrap();
        assert_eq!(
            invalid_prover.verify(),
            Err(vec![
                VerifyFailure::Permutation {
                    column: (Any::Instance, 0).into(),
                    location: FailureLocation::OutsideRegion { row: 2 }
                },
                VerifyFailure::Permutation {
                    column: (Any::advice(), 5).into(),
                    location: FailureLocation::InRegion {
                        region: (16, "permute state").into(),
                        offset: 38
                    }
                }
            ])
        );
    }

    // Passing an invalid leaf hash in the instance column should fail the permutation check between the (valid) leaf hash added as part of the witness and the instance column leaf hash
    #[test]
    fn test_invalid_leaf_hash_as_instance() {
        let assets_sum = Fp::from(556863u64); // greater than liabilities sum (556862)

        let circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        // add invalid leaf hash in the instance column
        let invalid_leaf_hash = Fp::from(1000u64);

        let public_input = vec![
            invalid_leaf_hash,
            circuit.leaf_balance,
            circuit.root_hash,
            circuit.assets_sum,
        ];

        let invalid_prover = MockProver::run(9, &circuit, vec![public_input]).unwrap();

        assert_eq!(
            invalid_prover.verify(),
            Err(vec![
                VerifyFailure::Permutation {
                    column: (Any::advice(), 0).into(),
                    location: FailureLocation::InRegion {
                        region: (1, "merkle prove layer").into(),
                        offset: 0
                    }
                },
                VerifyFailure::Permutation {
                    column: (Any::Instance, 0).into(),
                    location: FailureLocation::OutsideRegion { row: 0 }
                },
            ])
        );
    }

    // Passing an invalid leaf balance as input for the witness generation should fail the permutation check between the computed root hash and the instance column root hash
    #[test]
    fn test_invalid_leaf_balance_as_witness() {
        let assets_sum = Fp::from(556863u64); // greater than liabilities sum (556862)

        let user_balance = Fp::from(11888u64);

        let mut circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        // invalid leaf balance
        circuit.leaf_hash = Fp::from(1000u64);

        let public_input = vec![
            circuit.leaf_hash,
            user_balance,
            circuit.root_hash,
            assets_sum,
        ];

        let invalid_prover = MockProver::run(9, &circuit, vec![public_input]).unwrap();
        assert_eq!(
            invalid_prover.verify(),
            Err(vec![
                VerifyFailure::Permutation {
                    column: (Any::Instance, 0).into(),
                    location: FailureLocation::OutsideRegion { row: 2 }
                },
                VerifyFailure::Permutation {
                    column: (Any::advice(), 5).into(),
                    location: FailureLocation::InRegion {
                        region: (16, "permute state").into(),
                        offset: 38
                    }
                }
            ])
        );
    }

    // Passing an invalid leaf balance in the instance column should fail the permutation check between the (valid) leaf balance added as part of the witness and the instance column leaf balance
    #[test]
    fn test_invalid_leaf_balance_as_instance() {
        let assets_sum = Fp::from(556863u64); // greater than liabilities sum (556862)

        let circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        // add invalid leaf balance in the instance column
        let invalid_leaf_balance = Fp::from(1000u64);

        let public_input = vec![
            circuit.leaf_hash,
            invalid_leaf_balance,
            circuit.root_hash,
            circuit.assets_sum,
        ];

        let invalid_prover = MockProver::run(9, &circuit, vec![public_input]).unwrap();

        assert_eq!(
            invalid_prover.verify(),
            Err(vec![
                VerifyFailure::Permutation {
                    column: (Any::advice(), 1).into(),
                    location: FailureLocation::InRegion {
                        region: (1, "merkle prove layer").into(),
                        offset: 0
                    }
                },
                VerifyFailure::Permutation {
                    column: (Any::Instance, 0).into(),
                    location: FailureLocation::OutsideRegion { row: 1 }
                },
            ])
        );
    }

    // Passing a non binary index should fail the bool constraint check and the permutation check between the computed root hash and the instance column root hash
    #[test]
    fn test_non_binary_index() {
        let assets_sum = Fp::from(556863u64); // greater than liabilities sum (556862)

        let mut circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        // invalidate path index inside the circuit
        circuit.path_indices[0] = Fp::from(2);

        let public_input = vec![
            circuit.leaf_hash,
            circuit.leaf_balance,
            circuit.root_hash,
            circuit.assets_sum,
        ];

        let invalid_prover = MockProver::run(9, &circuit, vec![public_input]).unwrap();

        assert_eq!(
            invalid_prover.verify(),
            Err(vec![
                VerifyFailure::ConstraintNotSatisfied {
                    constraint: ((0, "bool constraint").into(), 0, "").into(),
                    location: FailureLocation::InRegion {
                        region: (1, "merkle prove layer").into(),
                        offset: 0
                    },
                    cell_values: vec![(((Any::advice(), 4).into(), 0).into(), "0x2".to_string()),]
                },
                VerifyFailure::Permutation {
                    column: (Any::Instance, 0).into(),
                    location: FailureLocation::OutsideRegion { row: 2 }
                },
                VerifyFailure::Permutation {
                    column: (Any::advice(), 5).into(),
                    location: FailureLocation::InRegion {
                        region: (16, "permute state").into(),
                        offset: 38
                    }
                }
            ])
        );
    }

    // Swapping the indices should fail the permutation check between the computed root hash and the instance column root hash
    #[test]
    fn test_swapping_index() {
        let assets_sum = Fp::from(556863u64); // greater than liabilities sum (556862)

        let mut circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        // swap indices
        circuit.path_indices[0] = Fp::from(1);

        let public_input = vec![
            circuit.leaf_hash,
            circuit.leaf_balance,
            circuit.root_hash,
            circuit.assets_sum,
        ];

        let invalid_prover = MockProver::run(9, &circuit, vec![public_input]).unwrap();

        assert_eq!(
            invalid_prover.verify(),
            Err(vec![
                VerifyFailure::Permutation {
                    column: (Any::Instance, 0).into(),
                    location: FailureLocation::OutsideRegion { row: 2 }
                },
                VerifyFailure::Permutation {
                    column: (Any::advice(), 5).into(),
                    location: FailureLocation::InRegion {
                        region: (16, "permute state").into(),
                        offset: 38
                    }
                }
            ])
        );
    }

    // Passing an assets sum that is less than the liabilities sum should fail the lessThan constraint check
    #[test]
    fn test_is_not_less_than() {
        let less_than_assets_sum = Fp::from(556861u64); // less than liabilities sum (556862)

        let circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
            less_than_assets_sum,
            "src/merkle_sum_tree/csv/entry_16.csv",
        );

        let public_input = vec![
            circuit.leaf_hash,
            circuit.leaf_balance,
            circuit.root_hash,
            circuit.assets_sum,
        ];

        let invalid_prover = MockProver::run(9, &circuit, vec![public_input]).unwrap();

        assert_eq!(
            invalid_prover.verify(),
            Err(vec![VerifyFailure::ConstraintNotSatisfied {
                constraint: (
                    (
                        7,
                        "verifies that `check` from current config equal to is_lt from LtChip"
                    )
                        .into(),
                    0,
                    ""
                )
                    .into(),
                location: FailureLocation::InRegion {
                    region: (18, "enforce sum to be less than total assets").into(),
                    offset: 0
                },
                cell_values: vec![
                    (((Any::advice(), 2).into(), 0).into(), "1".to_string()),
                    // The zero means that is not less than
                    (((Any::advice(), 11).into(), 0).into(), "0".to_string())
                ]
            }])
        );

        assert!(invalid_prover.verify().is_err());
    }

    use crate::circuits::ecdsa::EcdsaVerifyCircuit;
    use ecc::maingate::{big_to_fe, fe_to_big};
    use halo2_proofs::arithmetic::{CurveAffine, Field};
    use halo2_proofs::halo2curves::{group::Curve, secp256k1::Secp256k1Affine as Secp256k1};

    fn mod_n(x: <Secp256k1 as CurveAffine>::Base) -> <Secp256k1 as CurveAffine>::ScalarExt {
        let x_big = fe_to_big(x);
        big_to_fe(x_big)
    }

    #[test]
    fn test_ecdsa_valid_verifier() {
        let g = Secp256k1::generator();

        // Generate a key pair (sk, pk)
        // sk is a random scalar (exists within the scalar field, which is the order of the group generated by g
        // Note that the scalar field is different from the prime field of the curve.
        // pk is a point on the curve
        let sk = <Secp256k1 as CurveAffine>::ScalarExt::random(OsRng);

        let public_key = (g * sk).to_affine();

        let msg_hash = <Secp256k1 as CurveAffine>::ScalarExt::random(OsRng);

        // Draw arandomness -> k is also a scalar living in the order of the group generated by generator point g.
        let k = <Secp256k1 as CurveAffine>::ScalarExt::random(OsRng);
        let k_inv = k.invert().unwrap();

        let r_point = (g * k).to_affine().coordinates().unwrap();
        let x = r_point.x();

        // perform r mod n to ensure that r is a valid scalar
        let r = mod_n(*x);

        let s = k_inv * (msg_hash + (r * sk));

        // Sanity check. Ensure we construct a valid signature. So lets verify it
        {
            let s_inv = s.invert().unwrap();
            let u_1 = msg_hash * s_inv;
            let u_2 = r * s_inv;
            let r_point = ((g * u_1) + (public_key * u_2))
                .to_affine()
                .coordinates()
                .unwrap();
            let x_candidate = r_point.x();
            let r_candidate = mod_n(*x_candidate);
            assert_eq!(r, r_candidate);
        }

        let instance = vec![vec![]];

        let circuit = EcdsaVerifyCircuit::init(public_key, r, s, msg_hash);

        let valid_prover = MockProver::run(18, &circuit, instance).unwrap();

        valid_prover.assert_satisfied();
    }
}

#[cfg(feature = "dev-graph")]
#[test]
fn print_merkle_sum_tree() {
    use plotters::prelude::*;

    let assets_sum = Fp::from(556863u64); // greater than liabilities sum (556862)

    let circuit = MerkleSumTreeCircuit::init_from_assets_and_path(
        assets_sum,
        "src/merkle_sum_tree/csv/entry_16.csv",
    );

    let root =
        BitMapBackend::new("prints/merkle-sum-tree-layout.png", (2048, 16384)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root
        .titled("Merkle Sum Tree Layout", ("sans-serif", 60))
        .unwrap();

    halo2_proofs::dev::CircuitLayout::default()
        .render(8, &circuit, &root)
        .unwrap();
}
