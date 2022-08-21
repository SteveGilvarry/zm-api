import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsUpdateManyMutationInput } from './snapshots-update-many-mutation.input';
import { Type } from 'class-transformer';
import { SnapshotsWhereInput } from './snapshots-where.input';

@ArgsType()
export class UpdateManySnapshotsArgs {

    @Field(() => SnapshotsUpdateManyMutationInput, {nullable:false})
    @Type(() => SnapshotsUpdateManyMutationInput)
    data!: SnapshotsUpdateManyMutationInput;

    @Field(() => SnapshotsWhereInput, {nullable:true})
    @Type(() => SnapshotsWhereInput)
    where?: SnapshotsWhereInput;
}
