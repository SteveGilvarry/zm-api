import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesWhereUniqueInput } from './frames-where-unique.input';
import { FramesCreateInput } from './frames-create.input';
import { FramesUpdateInput } from './frames-update.input';

@ArgsType()
export class UpsertOneFramesArgs {

    @Field(() => FramesWhereUniqueInput, {nullable:false})
    where!: FramesWhereUniqueInput;

    @Field(() => FramesCreateInput, {nullable:false})
    create!: FramesCreateInput;

    @Field(() => FramesUpdateInput, {nullable:false})
    update!: FramesUpdateInput;
}
