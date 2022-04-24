import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsUpdateManyMutationInput } from './snapshots-update-many-mutation.input';
import { SnapshotsWhereInput } from './snapshots-where.input';

@ArgsType()
export class UpdateManySnapshotsArgs {

    @Field(() => SnapshotsUpdateManyMutationInput, {nullable:false})
    data!: SnapshotsUpdateManyMutationInput;

    @Field(() => SnapshotsWhereInput, {nullable:true})
    where?: SnapshotsWhereInput;
}
