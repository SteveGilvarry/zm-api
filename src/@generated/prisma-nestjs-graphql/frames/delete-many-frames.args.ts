import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesWhereInput } from './frames-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyFramesArgs {

    @Field(() => FramesWhereInput, {nullable:true})
    @Type(() => FramesWhereInput)
    where?: FramesWhereInput;
}
