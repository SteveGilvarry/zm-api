import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesUpdateInput } from './frames-update.input';
import { FramesWhereUniqueInput } from './frames-where-unique.input';

@ArgsType()
export class UpdateOneFramesArgs {

    @Field(() => FramesUpdateInput, {nullable:false})
    data!: FramesUpdateInput;

    @Field(() => FramesWhereUniqueInput, {nullable:false})
    where!: FramesWhereUniqueInput;
}
