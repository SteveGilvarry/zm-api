import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_OutputContainer } from './monitors-output-container.enum';

@InputType()
export class NullableEnumMonitors_OutputContainerFieldUpdateOperationsInput {

    @Field(() => Monitors_OutputContainer, {nullable:true})
    set?: keyof typeof Monitors_OutputContainer;
}
