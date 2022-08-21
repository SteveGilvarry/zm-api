import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsCreateManyInput } from './snapshots-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManySnapshotsArgs {

    @Field(() => [SnapshotsCreateManyInput], {nullable:false})
    @Type(() => SnapshotsCreateManyInput)
    data!: Array<SnapshotsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
