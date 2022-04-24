import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsUpdateInput } from './snapshots-update.input';
import { SnapshotsWhereUniqueInput } from './snapshots-where-unique.input';

@ArgsType()
export class UpdateOneSnapshotsArgs {

    @Field(() => SnapshotsUpdateInput, {nullable:false})
    data!: SnapshotsUpdateInput;

    @Field(() => SnapshotsWhereUniqueInput, {nullable:false})
    where!: SnapshotsWhereUniqueInput;
}
