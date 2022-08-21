import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsUpdateInput } from './snapshots-update.input';
import { Type } from 'class-transformer';
import { SnapshotsWhereUniqueInput } from './snapshots-where-unique.input';

@ArgsType()
export class UpdateOneSnapshotsArgs {

    @Field(() => SnapshotsUpdateInput, {nullable:false})
    @Type(() => SnapshotsUpdateInput)
    data!: SnapshotsUpdateInput;

    @Field(() => SnapshotsWhereUniqueInput, {nullable:false})
    @Type(() => SnapshotsWhereUniqueInput)
    where!: SnapshotsWhereUniqueInput;
}
