import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsWhereUniqueInput } from './snapshots-where-unique.input';
import { SnapshotsCreateInput } from './snapshots-create.input';
import { SnapshotsUpdateInput } from './snapshots-update.input';

@ArgsType()
export class UpsertOneSnapshotsArgs {

    @Field(() => SnapshotsWhereUniqueInput, {nullable:false})
    where!: SnapshotsWhereUniqueInput;

    @Field(() => SnapshotsCreateInput, {nullable:false})
    create!: SnapshotsCreateInput;

    @Field(() => SnapshotsUpdateInput, {nullable:false})
    update!: SnapshotsUpdateInput;
}
