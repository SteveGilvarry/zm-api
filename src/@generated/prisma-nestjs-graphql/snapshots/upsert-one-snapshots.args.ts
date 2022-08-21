import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsWhereUniqueInput } from './snapshots-where-unique.input';
import { Type } from 'class-transformer';
import { SnapshotsCreateInput } from './snapshots-create.input';
import { SnapshotsUpdateInput } from './snapshots-update.input';

@ArgsType()
export class UpsertOneSnapshotsArgs {

    @Field(() => SnapshotsWhereUniqueInput, {nullable:false})
    @Type(() => SnapshotsWhereUniqueInput)
    where!: SnapshotsWhereUniqueInput;

    @Field(() => SnapshotsCreateInput, {nullable:false})
    @Type(() => SnapshotsCreateInput)
    create!: SnapshotsCreateInput;

    @Field(() => SnapshotsUpdateInput, {nullable:false})
    @Type(() => SnapshotsUpdateInput)
    update!: SnapshotsUpdateInput;
}
